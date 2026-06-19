//! Witnex demo CLI.
//!
//! ```text
//! witnex demo summarize "<text>"   # run the agent, emit trace + proof JSON
//! witnex verify <proof.json>       # verify a proof bundle (next slice)
//! ```
//!
//! Phase 1 uses the in-process [`MockBackend`](witnex_core::llm::MockBackend) —
//! no network or API key required. Proof generation / verification land in the
//! Risc0 slice; for now the emitted bundle carries an empty placeholder proof.

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context as _;
use clap::{Parser, Subcommand};
use witnex_core::llm::{LlmBackend, LlmRequest, MockBackend};
use witnex_core::{ExecutionTrace, Nonce, Timestamp};
use witnex_prover::{Proof, ProofBundle};

/// Fixed prompt template for the summarize demo. The trace commits to this
/// template (its hash), separately from the raw input.
const SUMMARIZE_TEMPLATE: &str = "Summarize this in one sentence: {input}";

/// System prompt sent to the backend (not part of the committed template).
const SUMMARIZE_SYSTEM: &str =
    "You are a helpful assistant that writes concise one-sentence summaries.";

#[derive(Parser)]
#[command(
    name = "witnex",
    version,
    about = "Verifiable AI agent framework — demo CLI"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run a demo agent.
    Demo {
        #[command(subcommand)]
        demo: DemoCommand,
    },
    /// Verify a proof bundle (implemented in the Risc0 slice).
    Verify {
        /// Path to a proof bundle JSON file.
        path: PathBuf,
    },
}

#[derive(Subcommand)]
enum DemoCommand {
    /// Summarize text and emit a committed execution trace as JSON.
    Summarize {
        /// The text to summarize.
        text: String,
        /// Write the JSON bundle to this file instead of stdout.
        #[arg(short, long)]
        out: Option<PathBuf>,
    },
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::Demo {
            demo: DemoCommand::Summarize { text, out },
        } => run_summarize(&text, out.as_deref()),
        Command::Verify { .. } => {
            eprintln!("witnex verify: implemented in the next slice (Risc0 proof verification).");
            std::process::exit(2);
        }
    }
}

fn run_summarize(text: &str, out: Option<&Path>) -> anyhow::Result<()> {
    // Phase 1 backend: deterministic, offline mock.
    let summary = format!(
        "This text ({} chars) summarized in one sentence.",
        text.len()
    );
    let backend = MockBackend::new(summary);

    let filled_prompt = SUMMARIZE_TEMPLATE.replace("{input}", text);
    let request = LlmRequest::single_turn(SUMMARIZE_SYSTEM, filled_prompt, 256);
    let response =
        pollster::block_on(backend.complete(&request)).context("LLM backend completion failed")?;

    let trace = ExecutionTrace::commit(
        text,
        SUMMARIZE_TEMPLATE,
        response.model.clone(),
        &response.content,
        Vec::new(), // no tool calls in this demo
        Timestamp(unix_millis()?),
        Nonce(random_nonce()?),
    );

    // Risc0 proving lands next; emit a structurally complete bundle with an
    // empty placeholder proof for now.
    let bundle = ProofBundle {
        trace,
        proof: Proof { bytes: Vec::new() },
    };
    let json = serde_json::to_string_pretty(&bundle).context("serializing bundle")?;

    eprintln!("model:      {}", bundle.trace.model_id.0);
    eprintln!("output:     {}", response.content);
    eprintln!("commitment: {}", bundle.trace.commitment());

    match out {
        Some(path) => {
            std::fs::write(path, &json).with_context(|| format!("writing {}", path.display()))?;
            eprintln!("wrote bundle to {}", path.display());
        }
        None => println!("{json}"),
    }
    Ok(())
}

/// Current Unix time in milliseconds.
fn unix_millis() -> anyhow::Result<u64> {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock before Unix epoch")?
        .as_millis();
    Ok(millis as u64)
}

/// 32 bytes of OS randomness for the per-trace nonce.
fn random_nonce() -> anyhow::Result<[u8; 32]> {
    let mut buf = [0u8; 32];
    getrandom::getrandom(&mut buf).context("reading OS randomness for nonce")?;
    Ok(buf)
}
