/**
 * Mock AI Proxy — standalone Node.js HTTP server for Playwright E2E tests.
 *
 * Runs on port 11436 (configurable via PORT env var).
 * Implements the same POST /v1/propose contract as the real Rust ai-proxy,
 * but returns canned CommandEnvelope[] responses keyed by prompt patterns.
 *
 * Usage:
 *   node mock-ai-proxy.mjs
 *   PORT=11437 node mock-ai-proxy.mjs
 *
 * Responses are configured via the `RESPONSES` map below.
 * If a prompt matches a pattern key, the matching canned response is returned.
 * Otherwise, the DEFAULT_RESPONSE is returned.
 */

import { createServer } from "node:http";

const PORT = Number(process.env.PORT ?? 11436);

// ─── Canned responses ─────────────────────────────────────────────────────────

const COMMAND_ENVELOPES = {
  create_entity: {
    command: { type: "CreateEntity", id: "ent_ai_001", name: "AI Sprite" },
    metadata: {
      authorship: "agent:gpt-4o",
      timestamp: Date.now(),
      rationale: "Created a new sprite entity named 'AI Sprite'",
      model: "gpt-4o",
    },
  },
  set_field: {
    command: {
      type: "SetComponentField",
      entity_id: "ent_ai_001",
      type_id: "editor.Sprite2D",
      // editor.Sprite2D has fields {asset, color, anchor}. The `color`
      // field is a Color object {r, g, b, a}. To "update alpha", the
      // field path must be "color.a" (NOT "color.alpha").
      field_path: "color.a",
      value: 0.5,
    },
    metadata: {
      authorship: "agent:gpt-4o",
      timestamp: Date.now(),
      rationale: "Set color.a to 0.5",
      model: "gpt-4o",
    },
  },
};

/** Response map: prompt substring → ProposeResponse body */
const RESPONSES = new Map([
  [
    "create sprite",
    {
      // Hito 5 followups (v0.77.1): wrap the CreateEntity + SetComponentField
      // in a single Batch envelope so the frontend creates 1 Proposal
      // (instead of 2 separate Proposals). The inner CreateEntity must
      // include the editor.Sprite2D component instance, otherwise the
      // SetComponentField that follows would fail with "Unknown schema"
      // because the entity has no Sprite2D component attached yet.
      commands: [
        {
          command: {
            type: "Batch",
            label: "Create AI Sprite",
            commands: [
              {
                ...COMMAND_ENVELOPES.create_entity.command,
                components: [
                  // editor.Sprite2D (not editor.Transform2D) because the
                  // SetComponentField that follows references editor.Sprite2D.
                  // The Rust processor rejects SetComponentField if the
                  // entity has no component with that type_id attached.
                  {
                    type_id: "editor.Sprite2D",
                    values: {
                      asset: "",
                      color: { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
                      anchor: "Center",
                    },
                  },
                ],
              },
              COMMAND_ENVELOPES.set_field.command,
            ],
          },
          metadata: {
            authorship: "agent:gpt-4o",
            timestamp: Date.now(),
            rationale: "Created a new sprite entity named 'AI Sprite'",
            model: "gpt-4o",
          },
        },
      ],
      rationale: "Created a new sprite entity named 'AI Sprite'",
      model: "gpt-4o",
    },
  ],
  [
    "add enemy",
    {
      commands: [
        {
          command: { type: "CreateEntity", id: "ent_ai_002", name: "Enemy" },
          metadata: {
            authorship: "agent:gpt-4o",
            timestamp: Date.now(),
            rationale: "Created enemy entity",
            model: "gpt-4o",
          },
        },
      ],
      rationale: "Created an enemy entity",
      model: "gpt-4o",
    },
  ],
  [
    "default",
    {
      commands: [
        {
          command: { type: "CreateEntity", id: "ent_ai_003", name: "Generic Entity" },
          metadata: {
            authorship: "agent:gpt-4o",
            timestamp: Date.now(),
            rationale: "Generic suggestion",
            model: "gpt-4o",
          },
        },
      ],
      rationale: "Added a generic entity",
      model: "gpt-4o",
    },
  ],
]);

/** Look up a response by prompt substring match */
function lookupResponse(prompt) {
  const lower = prompt.toLowerCase();
  for (const [pattern, response] of RESPONSES) {
    if (pattern !== "default" && lower.includes(pattern)) {
      return response;
    }
  }
  return RESPONSES.get("default");
}

// ─── Hito 4 Order 6: code-aware AI patterns ───────────────────────────────────

/**
 * Build a code-aware response based on the incoming request context.
 * If source_files are present in the request, the mock uses the first
 * file's content as the basis for WriteSourceFile proposals.
 */
function buildCodeAwareResponse(prompt, body) {
  const lower = prompt.toLowerCase();
  const sourceFiles = body?.source_files ?? [];
  const firstFile = sourceFiles[0];

  // "create source file" / "create .rs" / "create .toml"
  if (lower.includes("source file") || lower.includes("create .rs") || lower.includes("create .toml")) {
    const isToml = lower.includes("toml");
    return {
      commands: [
        {
          command: {
            type: "CreateSourceFile",
            path: isToml ? "Cargo.toml" : "src/ai_generated.rs",
            name: isToml ? "Cargo.toml" : "ai_generated.rs",
            content: isToml
              ? "[package]\nname = \"ai_generated\"\nversion = \"0.1.0\"\n"
              : "// Auto-generated by AI\nfn ai_generated() {\n    println!(\"Hello from AI\");\n}\n",
          },
          metadata: {
            authorship: "agent:gpt-4o",
            timestamp: Date.now(),
            rationale: "Created new source file",
            model: "gpt-4o",
          },
        },
      ],
      rationale: "Created a new source file",
      model: "gpt-4o",
    };
  }

  // "write function" / "add method" / "modify .rs"
  if (lower.includes("write function") || lower.includes("add method") || lower.includes("modify .rs")) {
    const fileId = firstFile?.id ?? "src_unknown";
    const newContent = firstFile
      ? `${firstFile.content}\n\n// AI-added function\nfn ai_added() {}\n`
      : "fn ai_added() {}\n";
    return {
      commands: [
        {
          command: {
            type: "WriteSourceFile",
            id: fileId,
            content: newContent,
          },
          metadata: {
            authorship: "agent:gpt-4o",
            timestamp: Date.now(),
            rationale: "Added a new function to the source file",
            model: "gpt-4o",
          },
        },
      ],
      rationale: "Wrote new function to existing source file",
      model: "gpt-4o",
    };
  }

  // "logic graph" / "connect nodes"
  if (lower.includes("logic graph") || lower.includes("connect nodes")) {
    return {
      commands: [
        {
          command: {
            type: "Batch",
            label: "Logic graph update",
            commands: [
              { type: "CreateEntity", id: "ent_ai_logic_001", name: "LogicNode1" },
            ],
          },
          metadata: {
            authorship: "agent:gpt-4o",
            timestamp: Date.now(),
            rationale: "Added a logic node (mock — full logic graph support deferred to v1.1)",
            model: "gpt-4o",
          },
        },
      ],
      rationale: "Updated logic graph (mock)",
      model: "gpt-4o",
    };
  }

  // "asset" / "scene asset"
  if (lower.includes("scene asset") || (lower.includes("asset") && !lower.includes("source"))) {
    return {
      commands: [
        {
          command: { type: "CreateEntity", id: "ent_ai_asset_001", name: "AssetRef" },
          metadata: {
            authorship: "agent:gpt-4o",
            timestamp: Date.now(),
            rationale: "Created entity referencing scene asset (mock)",
            model: "gpt-4o",
          },
        },
      ],
      rationale: "Created asset reference (mock)",
      model: "gpt-4o",
    };
  }

  // Hito 4 Order 7 (scene-component-authoring) — 3 new patterns
  if (
    lower.includes("scene component") ||
    lower.includes("create scene component") ||
    lower.includes("derive scenecomponent")
  ) {
    return {
      commands: [
        {
          command: {
            type: "CreateSceneComponent",
            schema: {
              type_id: "game.EnemyAI",
              display_name: "Enemy AI",
              fields: [
                { name: "aggression", field_type: "F32", default: 0.5, constraints: [] },
              ],
              exports_to_bevy: true,
              kind: "scene_component",
              bound_scene_asset_ref: "level1",
              auto_spawn: true,
            },
          },
          metadata: {
            authorship: "agent:gpt-4o",
            timestamp: Date.now(),
            rationale: "Created SceneComponent schema (mock)",
            model: "gpt-4o",
          },
        },
      ],
      rationale: "Created SceneComponent (mock)",
      model: "gpt-4o",
    };
  }

  if (lower.includes("update component") || lower.includes("update scene component fields")) {
    return {
      commands: [
        {
          command: {
            type: "UpdateSceneComponentFields",
            type_id: "game.EnemyAI",
            fields: [
              { name: "aggression", field_type: "F32", default: 0.8, constraints: [] },
            ],
          },
          metadata: {
            authorship: "agent:gpt-4o",
            timestamp: Date.now(),
            rationale: "Updated SceneComponent fields (mock)",
            model: "gpt-4o",
          },
        },
      ],
      rationale: "Updated SceneComponent (mock)",
      model: "gpt-4o",
    };
  }

  if (lower.includes("bind scene") || lower.includes("bind to schema")) {
    return {
      commands: [
        {
          command: {
            type: "BindSceneToSchema",
            type_id: "game.EnemyAI",
            scene_asset_id: "level1",
          },
          metadata: {
            authorship: "agent:gpt-4o",
            timestamp: Date.now(),
            rationale: "Bound schema to scene asset (mock)",
            model: "gpt-4o",
          },
        },
      ],
      rationale: "Bound SceneComponent (mock)",
      model: "gpt-4o",
    };
  }

  return null;
}

// ─── HTTP server ─────────────────────────────────────────────────────────────

function parseBody(req) {
  return new Promise((resolve, reject) => {
    let data = "";
    req.on("data", (chunk) => (data += chunk));
    req.on("end", () => {
      try {
        resolve(JSON.parse(data));
      } catch {
        reject(new Error("Invalid JSON"));
      }
    });
    req.on("error", reject);
  });
}

const server = createServer(async (req, res) => {
  // CORS headers
  res.setHeader("Access-Control-Allow-Origin", "*");
  res.setHeader("Access-Control-Allow-Methods", "POST, OPTIONS");
  res.setHeader("Access-Control-Allow-Headers", "Content-Type");

  if (req.method === "OPTIONS") {
    res.writeHead(204);
    res.end();
    return;
  }

  const url = new URL(req.url, `http://localhost:${PORT}`);

  // POST /v1/propose
  if (req.method === "POST" && url.pathname === "/v1/propose") {
    let body;
    try {
      body = await parseBody(req);
    } catch {
      res.writeHead(400, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ error: "Invalid JSON body" }));
      return;
    }

    const prompt = body?.prompt ?? "";

    // Simulate a small delay (50-150ms) like a real AI call
    await new Promise((r) => setTimeout(r, 50 + Math.floor(Math.random() * 100)));

    // Hito 4 Order 6: try code-aware response first (new patterns),
    // fall back to legacy pattern matching.
    const codeAware = buildCodeAwareResponse(prompt, body);
    const response = codeAware ?? lookupResponse(prompt);

    res.writeHead(200, { "Content-Type": "application/json" });
    res.end(JSON.stringify(response));
    return;
  }

  // GET /health
  if (req.method === "GET" && url.pathname === "/health") {
    res.writeHead(200, { "Content-Type": "application/json" });
    res.end(JSON.stringify({ status: "ok" }));
    return;
  }

  res.writeHead(404, { "Content-Type": "application/json" });
  res.end(JSON.stringify({ error: "Not found" }));
});

server.listen(PORT, () => {
  console.log(`[mock-ai-proxy] Listening on http://localhost:${PORT}`);
  console.log(
    `[mock-ai-proxy] Prompt patterns: ${[...RESPONSES.keys()]
      .filter((k) => k !== "default")
      .join(", ")}`
  );
});

server.on("error", (err) => {
  console.error(`[mock-ai-proxy] Server error: ${err.message}`);
  process.exit(1);
});
