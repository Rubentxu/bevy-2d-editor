//! OpenAI API client integration.

mod client;
mod function_calling;

pub use client::OpenAIClient;
pub use function_calling::CommandEnvelope;
