//! AI Proxy — local Rust HTTP proxy that holds the OpenAI API key server-side.
//!
//! Exposes `POST /v1/propose` which accepts `{ prompt, scene_snapshot, schemas }`
//! and returns command proposals from GPT-4o via function-calling.
//!
//! Usage:
//!     cargo run -p ai-proxy
//!
//! Environment variables:
//!     OPENAI_API_KEY   — required; OpenAI API key
//!     OPENAI_MODEL     — model name (default: gpt-4o)
//!     PORT             — port to bind (default: 11435)
//!     TOKEN_THRESHOLD  — max tokens before scene truncation (default: 10000)
//!     ALLOWED_ORIGINS  — comma-separated CORS origins (default: http://localhost:5173)

use clap::Parser;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use ai_proxy::config::AppConfig;
use ai_proxy::server::build_router;

#[derive(Debug, Parser)]
#[command(name = "ai-proxy")]
#[command(about = "AI-assisted editing proxy — holds OpenAI key server-side")]
struct Args {
    /// Port to listen on (overrides PORT env var).
    #[arg(short, long, env = "PORT")]
    port: Option<u16>,

    /// OpenAI model to use (overrides OPENAI_MODEL env var).
    #[arg(long, env = "OPENAI_MODEL")]
    model: Option<String>,

    /// Maximum tokens before scene truncation (overrides TOKEN_THRESHOLD env var).
    #[arg(long, env = "TOKEN_THRESHOLD")]
    token_threshold: Option<usize>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ai_proxy=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse CLI args
    let args = Args::parse();

    // Load config (OPENAI_API_KEY is required)
    let mut config = AppConfig::from_env().map_err(|e| {
        eprintln!("ERROR: {}", e);
        e
    })?;

    // CLI overrides env
    if let Some(p) = args.port {
        config.port = p;
    }
    if let Some(m) = args.model {
        config.model = m;
    }
    if let Some(t) = args.token_threshold {
        config.token_threshold = t;
    }

    let addr: SocketAddr = ([127, 0, 0, 1], config.port).into();

    let router = build_router(&config);

    tracing::info!("Starting AI proxy on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;

    Ok(())
}
