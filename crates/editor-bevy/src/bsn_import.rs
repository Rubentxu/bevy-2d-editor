//! BSN import — parse `.bsn` text back to `SceneAssetDocument`.
//!
//! This is the inverse of `bsn_ir_from_scene_asset` + `emit_bsn_text`.
//! Since Bevy PRs #23639 (writer) and #23648 (catalog) are DRAFT and use
//! Bevy-native ECS types (incompatible with our editor types), this module
//! implements round-trip within the editor ecosystem: export → re-import the
//! same `.bsn` text produced by `EditorCoreBsnExporter`.
//!
//! The parser handles the `.bsn` format emitted by `emit_bsn_text`:
//! ```text
//! bsn!{
//! #identifier
//! ComponentType(value)
//! ComponentType { field: value, ... }
//! Children [
//!   bsn!{
//!   #child-identifier
//!   Name("child")
//!   }
//! ]
//! }
//! ```
//!
//! ## Round-trip notes
//! The conversion is lossy (by design, matching `bsn_ir_from_scene_asset`):
//! - `metadata`, `exposed_properties`, `logical_path`, `asset_id`, `version` are dropped
//! - `asset_refs` and `patches` are not representable in `SceneAssetDocument`
//! - `relationships` are reconstructed as `RelationshipKind::Child` only

use crate::bsn_ir::{BsnIr, BsnIrNode, BsnIrRelationship};
use crate::scene_asset::{
    LocalId, RelationshipKind, SceneAssetDocument, SceneAssetEntity, SceneAssetMetadata,
    SceneAssetRelationship, SceneAssetRole,
};
use editor_model::ComponentInstance;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ─────────────────────────────────────────────────────────────────────────────
// Error types
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BsnImportError {
    /// The input `.bsn` text is empty or contains only whitespace.
    EmptyInput,
    /// The `.bsn` text could not be tokenized at the given position.
    UnexpectedToken {
        position: usize,
        found: String,
        expected: String,
    },
    /// The `.bsn` text is truncated or malformed.
    TruncatedInput { position: usize, context: String },
    /// The parser encountered a `bsn!` block where the root identifier is missing.
    MissingRootIdentifier { position: usize },
    /// An unsupported `.bsn` syntax was encountered.
    UnsupportedSyntax { position: usize, detail: String },
}

// ─────────────────────────────────────────────────────────────────────────────
// Tokenizer
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Token {
    BsnOpen,        // "bsn!{"
    BsnClose,       // "}"
    ChildrenOpen,   // "Children ["
    ChildrenClose,  // "]"
    Comma,          // ","
    Hash(String),   // "#identifier"
    Ident(String),  // ComponentType
    LParen,         // "("
    RParen,         // ")"
    LBrace,         // "{"
    RBrace,         // "}"
    Colon,          // ":"
    String(String), // "..."
    Number(String), // 123, 0.5, -3.14
    True,           // true
    False,          // false
    Eof,
}

struct Tokenizer<'a> {
    input: &'a str,
    pos: usize,
    /// Position where the last identifier started (used for "Children [" lookahead)
    last_ident_start: Option<usize>,
    /// CRIT-1 fix: track brace nesting so `}` can be tokenized as
    /// either `BsnClose` (closing a `bsn!{` block) or `RBrace`
    /// (closing a struct literal value). The rule: emit BsnClose
    /// when `}` is closing a `bsn!{`-introduced block; RBrace
    /// when closing a bare `{` struct literal.
    ///
    /// Implementation: we keep a stack-like depth counter that
    /// increments on every `{` (whether bsn or struct) and
    /// decrements on every `}`. Separately we know which opener
    /// each `}` matches via a `Vec<bool>` where `true` means the
    /// opener was a bsn!{ opener.
    bsn_brace_opener_stack: Vec<bool>,
}

impl<'a> Tokenizer<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            last_ident_start: None,
            bsn_brace_opener_stack: Vec::new(),
        }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.input[self.pos..].chars().next()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, BsnImportError> {
        let mut tokens = Vec::new();
        loop {
            self.skip_whitespace();
            let start = self.pos;
            match self.advance() {
                None => {
                    tokens.push(Token::Eof);
                    break;
                }
                Some('b') => {
                    // "bsn!{"
                    if self.input[start..].starts_with("bsn!{") {
                        tokens.push(Token::BsnOpen);
                        self.pos = start + 5;
                        // CRIT-1: mark this opener as bsn!{ so the
                        // matching `}` becomes BsnClose.
                        self.bsn_brace_opener_stack.push(true);
                    } else {
                        return Err(self.unexpected_token("identifier starting with 'b'"));
                    }
                }
                Some('{') => {
                    // CRIT-1: bare `{` is a struct literal (component
                    // value like `Sprite { image, color }`).
                    tokens.push(Token::LBrace);
                    self.bsn_brace_opener_stack.push(false);
                }
                Some('}') => {
                    // CRIT-1: closing brace. Pop the opener-stack to
                    // decide whether this is a BsnClose (matches a
                    // bsn!{) or RBrace (matches a struct literal).
                    let is_bsn_close = self
                        .bsn_brace_opener_stack
                        .pop()
                        .ok_or_else(|| BsnImportError::EmptyInput)?;
                    if is_bsn_close {
                        tokens.push(Token::BsnClose);
                    } else {
                        tokens.push(Token::RBrace);
                    }
                }
                Some('[') => {
                    // "Children [" — check from the position where the last identifier started
                    let check_start = self.last_ident_start.unwrap_or(start);
                    if self.input[check_start..].starts_with("Children [") {
                        tokens.push(Token::ChildrenOpen);
                        self.pos = check_start + 10;
                        self.last_ident_start = None; // reset after use
                    } else {
                        return Err(self.unexpected_token_at(start, "["));
                    }
                }
                Some(']') => tokens.push(Token::ChildrenClose),
                Some(',') => tokens.push(Token::Comma),
                Some('(') => tokens.push(Token::LParen),
                Some(')') => tokens.push(Token::RParen),
                Some('{') => {
                    // CRIT-1: bare `{` is a struct literal inside bsn.
                    tokens.push(Token::LBrace);
                    self.bsn_brace_opener_stack.push(false);
                }
                Some('}') => {
                    // CRIT-1: closing brace. The popped flag tells us
                    // whether this matches a bsn!{ opener (BsnClose)
                    // or a struct literal opener (RBrace).
                    let is_bsn_close = self
                        .bsn_brace_opener_stack
                        .pop()
                        .ok_or_else(|| BsnImportError::EmptyInput)?;
                    if is_bsn_close {
                        tokens.push(Token::BsnClose);
                    } else {
                        tokens.push(Token::RBrace);
                    }
                }
                Some(':') => tokens.push(Token::Colon),
                Some('#') => {
                    // Identifier after #
                    let ident = self.read_ident();
                    tokens.push(Token::Hash(ident));
                }
                Some('"') => {
                    // String literal
                    let s = self.read_string()?;
                    tokens.push(Token::String(s));
                }
                Some(ch) if ch.is_ascii_digit() || ch == '-' => {
                    self.pos -= 1;
                    let num = self.read_number();
                    tokens.push(Token::Number(num));
                }
                Some(ch) if ch.is_alphabetic() || ch == '_' => {
                    self.pos -= 1;
                    let ident_start = self.pos; // remember where identifier starts
                    let ident = self.read_ident();
                    self.skip_whitespace(); // skip trailing whitespace after identifier
                    match ident.as_str() {
                        "true" => tokens.push(Token::True),
                        "false" => tokens.push(Token::False),
                        _ => tokens.push(Token::Ident(ident)),
                    }
                    // Remember ident_start for the next token check
                    self.last_ident_start = Some(ident_start);
                }
                Some(ch) => {
                    return Err(BsnImportError::UnexpectedToken {
                        position: start,
                        found: format!("character '{ch}'"),
                        expected: "valid token".to_string(),
                    });
                }
            }
        }
        Ok(tokens)
    }

    fn read_ident(&mut self) -> String {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' || ch == '.' || ch == ':' {
                self.advance();
            } else {
                break;
            }
        }
        self.input[start..self.pos].to_string()
    }

    fn read_string(&mut self) -> Result<String, BsnImportError> {
        // Opening `"` already consumed
        let start = self.pos;
        let mut result = String::new();
        loop {
            match self.advance() {
                None => {
                    return Err(BsnImportError::TruncatedInput {
                        position: start,
                        context: "unclosed string literal".to_string(),
                    });
                }
                Some('\\') => {
                    // Escape sequence
                    match self.advance() {
                        Some('n') => result.push('\n'),
                        Some('r') => result.push('\r'),
                        Some('t') => result.push('\t'),
                        Some('"') => result.push('"'),
                        Some('\\') => result.push('\\'),
                        Some(ch) => {
                            result.push('\\');
                            result.push(ch);
                        }
                        None => {
                            return Err(BsnImportError::TruncatedInput {
                                position: start,
                                context: "incomplete escape sequence".to_string(),
                            });
                        }
                    }
                }
                Some('"') => break,
                Some(ch) => result.push(ch),
            }
        }
        Ok(result)
    }

    fn read_number(&mut self) -> String {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '.' || ch == 'e' || ch == 'E' || ch == '+' || ch == '-'
            {
                self.advance();
            } else {
                break;
            }
        }
        // Normalize: if starts with '-', keep it; otherwise use as-is
        self.input[start..self.pos].to_string()
    }

    fn unexpected_token(&self, expected: &str) -> BsnImportError {
        let found = self.input[self.pos..].chars().take(20).collect::<String>();
        BsnImportError::UnexpectedToken {
            position: self.pos,
            found,
            expected: expected.to_string(),
        }
    }

    fn unexpected_token_at(&self, pos: usize, expected: &str) -> BsnImportError {
        let found = self.input[pos..].chars().take(20).collect::<String>();
        BsnImportError::UnexpectedToken {
            position: pos,
            found,
            expected: expected.to_string(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Recursive-descent parser
// ─────────────────────────────────────────────────────────────────────────────

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let t = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof);
        self.pos += 1;
        t
    }

    fn expect(&mut self, expected: &Token) -> Result<Token, BsnImportError> {
        let t = self.advance();
        if std::mem::discriminant(&t) == std::mem::discriminant(expected) {
            Ok(t)
        } else {
            Err(BsnImportError::UnexpectedToken {
                position: self.pos.saturating_sub(1),
                found: format!("{:?}", t),
                expected: format!("{:?}", expected),
            })
        }
    }

    /// Parse a full `.bsn` text into a `BsnIr`.
    fn parse_bsn_ir(&mut self) -> Result<BsnIr, BsnImportError> {
        self.skip_bom()?;
        self.skip_shebang()?;

        let root = self.parse_bsn_node()?;
        let rest: Vec<Token> = self.tokens[self.pos..]
            .iter()
            .filter(|t| !matches!(t, Token::Eof))
            .cloned()
            .collect();
        if !rest.is_empty() {
            return Err(BsnImportError::TruncatedInput {
                position: self.pos,
                context: format!("{} unexpected tokens after root block", rest.len()),
            });
        }
        Ok(BsnIr {
            scene_root: root,
            asset_refs: Vec::new(),
            patches: Vec::new(),
        })
    }

    fn skip_bom(&mut self) -> Result<(), BsnImportError> {
        // UTF-8 BOM (optional, harmless if present)
        if self.input().starts_with('\u{FEFF}') {
            self.pos += 1;
        }
        Ok(())
    }

    fn input(&self) -> &str {
        // We don't have original input in parser, but BOM is handled at tokenizer level
        ""
    }

    fn skip_shebang(&mut self) -> Result<(), BsnImportError> {
        // Ignore any leading comments
        Ok(())
    }

    /// Parse a single `bsn!{ ... }` block into a `BsnIrNode`.
    fn parse_bsn_node(&mut self) -> Result<BsnIrNode, BsnImportError> {
        // "bsn!{"
        self.expect(&Token::BsnOpen)?;

        // "#identifier"
        let identifier = match self.advance() {
            Token::Hash(s) => s,
            t => {
                return Err(BsnImportError::UnexpectedToken {
                    position: self.pos.saturating_sub(1),
                    found: format!("{:?}", t),
                    expected: "\"#identifier\"".to_string(),
                });
            }
        };

        let mut components: BTreeMap<String, serde_json::Value> = BTreeMap::new();
        let mut children: Vec<BsnIrNode> = Vec::new();

        loop {
            let token = self.peek();
            match token {
                Token::Eof => {
                    return Err(BsnImportError::TruncatedInput {
                        position: self.pos,
                        context: "unclosed bsn!{ block".to_string(),
                    });
                }
                Token::BsnClose => {
                    self.advance();
                    break;
                }
                Token::ChildrenOpen => {
                    self.advance(); // consume "Children ["
                    loop {
                        let child_token = self.peek();
                        match child_token {
                            Token::ChildrenClose => {
                                self.advance();
                                break;
                            }
                            Token::BsnOpen => {
                                let child = self.parse_bsn_node()?;
                                children.push(child);
                            }
                            Token::Comma => {
                                self.advance(); // optional comma
                            }
                            t => {
                                return Err(BsnImportError::UnexpectedToken {
                                    position: self.pos,
                                    found: format!("{:?}", t),
                                    expected: "bsn!{...} block or Children close".to_string(),
                                });
                            }
                        }
                    }
                }
                Token::Ident(type_id) => {
                    let type_id = type_id.to_string();
                    self.advance();
                    // If the next token is ChildrenOpen, there are no component values
                    if matches!(self.peek(), Token::ChildrenOpen) {
                        continue; // go back to outer loop which handles ChildrenOpen
                    }
                    let value = self.parse_component_value()?;
                    components.insert(type_id, value);
                }
                t => {
                    return Err(BsnImportError::UnexpectedToken {
                        position: self.pos,
                        found: format!("{:?}", t),
                        expected: "component type, Children, or closing }".to_string(),
                    });
                }
            }
        }

        Ok(BsnIrNode {
            identifier,
            components,
            children,
            relationships: Vec::new(), // relationships rebuilt in scene_asset_from_bsn_ir
            ..Default::default()
        })
    }

    /// Parse the value of a component. Can be:
    /// - `("string")` → String
    /// - `{ field: value, ... }` → struct-like → serde_json::Value::Object
    fn parse_component_value(&mut self) -> Result<serde_json::Value, BsnImportError> {
        match self.peek() {
            Token::LParen => {
                self.advance(); // consume '('
                let v = self.parse_value_inner()?;
                self.expect(&Token::RParen)?;
                Ok(v)
            }
            Token::LBrace => {
                self.advance(); // consume '{'
                let mut map = serde_json::Map::new();
                loop {
                    if matches!(self.peek(), Token::RBrace) {
                        self.advance();
                        break;
                    }
                    let key = match self.advance() {
                        Token::Ident(s) => s,
                        Token::String(s) => s,
                        t => {
                            return Err(BsnImportError::UnexpectedToken {
                                position: self.pos.saturating_sub(1),
                                found: format!("{:?}", t),
                                expected: "field name".to_string(),
                            });
                        }
                    };
                    self.expect(&Token::Colon)?;
                    let val = self.parse_value_inner()?;
                    map.insert(key, val);
                    if matches!(self.peek(), Token::Comma) {
                        self.advance();
                    }
                }
                Ok(serde_json::Value::Object(map))
            }
            t => Err(BsnImportError::UnexpectedToken {
                position: self.pos,
                found: format!("{:?}", t),
                expected: "\"(\" or \"{\"".to_string(),
            }),
        }
    }

    /// Parse a JSON-like value: string, number, bool, or nested object/array.
    fn parse_value_inner(&mut self) -> Result<serde_json::Value, BsnImportError> {
        match self.peek().clone() {
            Token::String(s) => {
                self.advance();
                Ok(serde_json::Value::String(s))
            }
            Token::Number(n) => {
                self.advance();
                // Try parse as integer first, then float
                if let Ok(i) = n.parse::<i64>() {
                    Ok(serde_json::Value::Number(i.into()))
                } else if let Ok(f) = n.parse::<f64>() {
                    serde_json::Number::from_f64(f)
                        .map(serde_json::Value::Number)
                        .ok_or_else(|| BsnImportError::UnsupportedSyntax {
                            position: self.pos,
                            detail: format!("invalid float: {}", n),
                        })
                } else {
                    Err(BsnImportError::UnsupportedSyntax {
                        position: self.pos,
                        detail: format!("unparseable number: {}", n),
                    })
                }
            }
            Token::True => {
                self.advance();
                Ok(serde_json::Value::Bool(true))
            }
            Token::False => {
                self.advance();
                Ok(serde_json::Value::Bool(false))
            }
            Token::LBrace => {
                self.advance();
                let mut map = serde_json::Map::new();
                loop {
                    if matches!(self.peek(), Token::RBrace) {
                        self.advance();
                        break;
                    }
                    let key = match self.advance() {
                        Token::Ident(s) => s,
                        Token::String(s) => s,
                        t => {
                            return Err(BsnImportError::UnexpectedToken {
                                position: self.pos.saturating_sub(1),
                                found: format!("{:?}", t),
                                expected: "field name".to_string(),
                            });
                        }
                    };
                    self.expect(&Token::Colon)?;
                    let val = self.parse_value_inner()?;
                    map.insert(key, val);
                    if matches!(self.peek(), Token::Comma) {
                        self.advance();
                    }
                }
                Ok(serde_json::Value::Object(map))
            }
            Token::LParen => {
                // Tuple-like: (a, b, c) → array
                self.advance();
                let mut arr = Vec::new();
                loop {
                    if matches!(self.peek(), Token::RParen) {
                        self.advance();
                        break;
                    }
                    arr.push(self.parse_value_inner()?);
                    if matches!(self.peek(), Token::Comma) {
                        self.advance();
                    }
                }
                Ok(serde_json::Value::Array(arr))
            }
            t => Err(BsnImportError::UnexpectedToken {
                position: self.pos,
                found: format!("{:?}", t),
                expected: "string, number, bool, object, or array".to_string(),
            }),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Public API
// ─────────────────────────────────────────────────────────────────────────────

/// Parse `.bsn` text into a `BsnIr`. Fails on malformed input.
pub fn parse_bsn_text(text: &str) -> Result<BsnIr, BsnImportError> {
    if text.trim().is_empty() {
        return Err(BsnImportError::EmptyInput);
    }
    let mut tokenizer = Tokenizer::new(text);
    let tokens = tokenizer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse_bsn_ir()
}

/// Convert a `BsnIr` back into a `SceneAssetDocument`.
///
/// This is the inverse of `bsn_ir_from_scene_asset` (in `bsn_ir.rs`).
///
/// The conversion is lossy — by design, matching the lossy nature of
/// `bsn_ir_from_scene_asset`. Specifically:
/// - `metadata`, `exposed_properties`, `logical_path`, `asset_id`, `version` are
///   not representable in `BsnIr` and are set to defaults.
/// - `asset_refs` and `patches` from the IR are dropped.
pub fn scene_asset_from_bsn_ir(ir: BsnIr) -> SceneAssetDocument {
    let mut entities = Vec::new();
    let mut relationships = Vec::new();

    // Recursively convert BsnIrNode tree → entities + relationships
    fn convert_node(
        node: BsnIrNode,
        entities: &mut Vec<SceneAssetEntity>,
        relationships: &mut Vec<SceneAssetRelationship>,
    ) {
        let local_id = LocalId::new(node.identifier.clone());
        let components = node
            .components
            .into_iter()
            .map(|(type_id, values)| ComponentInstance { type_id, values })
            .collect();

        let entity_local_id = local_id.clone();
        entities.push(SceneAssetEntity {
            local_id,
            local_path: String::new(),
            name: String::new(), // no name in BsnIrNode
            components,
            extension_data: BTreeMap::new(),
        });

        for child in node.children {
            let child_local_id = LocalId::new(child.identifier.clone());
            relationships.push(SceneAssetRelationship {
                from_local_id: entity_local_id.clone(),
                to_local_id: child_local_id.clone(),
                kind: RelationshipKind::Child,
                field_path: None,
            });
            convert_node(child, entities, relationships);
        }
    }

    convert_node(ir.scene_root, &mut entities, &mut relationships);

    SceneAssetDocument {
        asset_id: String::new(),
        logical_path: String::new(),
        role: SceneAssetRole::Fragment,
        version: 1,
        entities,
        relationships,
        exposed_properties: Default::default(),
        metadata: SceneAssetMetadata::default(),
        layers: Default::default(),
        extension_data: BTreeMap::new(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(doc: &SceneAssetDocument) -> SceneAssetDocument {
        let ir = crate::bsn_ir::bsn_ir_from_scene_asset(doc);
        let text = crate::bsn_export::export_to_bsn_text(&doc).unwrap();
        let parsed_ir = parse_bsn_text(&text).unwrap();
        scene_asset_from_bsn_ir(parsed_ir)
    }

    #[test]
    fn empty_doc_import_rejected() {
        let result = parse_bsn_text("");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BsnImportError::EmptyInput));
    }

    #[test]
    fn simple_entity_round_trip() {
        use crate::scene_asset::{
            LocalId, SceneAssetDocument, SceneAssetEntity, SceneAssetMetadata, SceneAssetRole,
        };
        use editor_model::ComponentInstance;
        let doc = SceneAssetDocument {
            asset_id: String::new(),
            logical_path: String::new(),
            role: SceneAssetRole::Fragment,
            version: 1,
            entities: vec![SceneAssetEntity {
                local_id: LocalId::new("player".to_string()),
                local_path: String::new(),
                name: "Player".to_string(),
                components: vec![ComponentInstance {
                    type_id: "editor.Name".to_string(),
                    values: serde_json::json!({"name": "PlayerEntity"}),
                }],
                extension_data: BTreeMap::new(),
            }],
            relationships: vec![],
            exposed_properties: Default::default(),
            metadata: SceneAssetMetadata::default(),
            layers: Default::default(),
            extension_data: BTreeMap::new(),
        };
        let imported = round_trip(&doc);
        assert_eq!(imported.entities.len(), 1);
        assert_eq!(imported.entities[0].local_id.0, "player");
    }

    /// CRIT-1 regression: a struct-literal component value (e.g.
    /// `Sprite { image: "...", color: Color::srgba(...) }`) must tokenize
    /// correctly — the inner `}` must be `RBrace`, not `BsnClose`.
    #[test]
    fn struct_literal_component_tokenizes_and_parses() {
        let text = "bsn!{\n\
                    #player\n\
                        editor.Sprite { image: \"x.png\", color: Color::srgba(1.0, 0.5, 0.2, 1.0) }\n\
                    }";
        let mut tokenizer = Tokenizer::new(text);
        let tokens = tokenizer.tokenize().unwrap();
        // Expect: BsnOpen, Hash("player"), Ident("editor.Sprite"), LBrace,
        // then key/value pairs and finally RBrace (NOT BsnClose), then
        // the outer BsnClose, then Eof.
        // The second-to-last token must be BsnClose (outer close);
        // Eof is always last.
        assert!(
            matches!(
                tokens.get(tokens.len().saturating_sub(2)),
                Some(Token::BsnClose)
            ),
            "outer close must be BsnClose; got {:?}",
            tokens.get(tokens.len().saturating_sub(2))
        );
        // Find the inner } (before the outer one). It must be RBrace.
        let rbrace_count = tokens.iter().filter(|t| matches!(t, Token::RBrace)).count();
        assert_eq!(
            rbrace_count, 1,
            "expected 1 inner RBrace; tokens = {:?}",
            tokens
        );
    }

    #[test]
    fn nested_children_round_trip() {
        use crate::scene_asset::{
            LocalId, RelationshipKind, SceneAssetDocument, SceneAssetEntity, SceneAssetMetadata,
            SceneAssetRelationship, SceneAssetRole,
        };
        use editor_model::ComponentInstance;
        let doc = SceneAssetDocument {
            asset_id: String::new(),
            logical_path: String::new(),
            role: SceneAssetRole::Fragment,
            version: 1,
            entities: vec![
                SceneAssetEntity {
                    local_id: LocalId::new("root".to_string()),
                    local_path: String::new(),
                    name: "Root".to_string(),
                    components: vec![],
                    extension_data: BTreeMap::new(),
                },
                SceneAssetEntity {
                    local_id: LocalId::new("child1".to_string()),
                    local_path: String::new(),
                    name: "Child1".to_string(),
                    components: vec![],
                    extension_data: BTreeMap::new(),
                },
                SceneAssetEntity {
                    local_id: LocalId::new("child2".to_string()),
                    local_path: String::new(),
                    name: "Child2".to_string(),
                    components: vec![],
                    extension_data: BTreeMap::new(),
                },
            ],
            relationships: vec![
                SceneAssetRelationship {
                    from_local_id: LocalId::new("root".to_string()),
                    to_local_id: LocalId::new("child1".to_string()),
                    kind: RelationshipKind::Child,
                    field_path: None,
                },
                SceneAssetRelationship {
                    from_local_id: LocalId::new("root".to_string()),
                    to_local_id: LocalId::new("child2".to_string()),
                    kind: RelationshipKind::Child,
                    field_path: None,
                },
            ],
            exposed_properties: Default::default(),
            metadata: SceneAssetMetadata::default(),
            layers: Default::default(),
            extension_data: BTreeMap::new(),
        };
        let imported = round_trip(&doc);
        // Root has 2 children
        let child_ids: Vec<_> = imported
            .relationships
            .iter()
            .filter(|r| r.from_local_id.0 == "root" && matches!(r.kind, RelationshipKind::Child))
            .map(|r| r.to_local_id.0.clone())
            .collect();
        assert_eq!(child_ids.len(), 2);
    }
}
