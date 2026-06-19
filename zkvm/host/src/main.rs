//! `witnex-zkvm` — generate and verify real Risc0 proofs over Witnex bundles.
//!
//! ```text
//! witnex-zkvm prove  <bundle.json> [-o out]   # add a Risc0 receipt to the bundle
//! witnex-zkvm verify <bundle.json>            # check the receipt -> VERIFIED / INVALID
//! ```
//!
//! Pairs with the main `witnex` CLI: `witnex demo summarize ... --out b.json`
//! produces the (unproven) bundle; `witnex-zkvm prove b.json` fills in the
//! proof. Set `RISC0_DEV_MODE=1` for a fast dev receipt.

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use witnex_prover::ProofBundle;
use witnex_zkvm_host::{attach_proof, verify_bundle};

#[derive(Parser)]
#[command(name = "witnex-zkvm", version, about = "Witnex Risc0 prover / verifier")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate a Risc0 proof for a bundle's trace and embed it.
    Prove {
        /// Path to a bundle JSON (e.g. from `witnex demo summarize --out`).
        path: PathBuf,
        /// Output path (defaults to overwriting the input).
        #[arg(short, long)]
        out: Option<PathBuf>,
    },
    /// Verify a bundle's embedded Risc0 receipt.
    Verify {
        /// Path to a proven bundle JSON.
        path: PathBuf,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Command::Prove { path, out } => run_prove(&path, out.as_deref()),
        Command::Verify { path } => run_verify(&path),
    }
}

fn read_bundle(path: &Path) -> Result<ProofBundle> {
    let json =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&json).with_context(|| format!("parsing {}", path.display()))
}

fn run_prove(path: &Path, out: Option<&Path>) -> Result<()> {
    let bundle = read_bundle(path)?;
    eprintln!("proving trace (commitment {}) ...", bundle.trace.commitment());
    let proven = attach_proof(&bundle)?;
    let json = serde_json::to_string_pretty(&proven).context("serializing bundle")?;
    let dest = out.unwrap_or(path);
    std::fs::write(dest, &json).with_context(|| format!("writing {}", dest.display()))?;
    eprintln!(
        "wrote proof bundle to {} ({} proof bytes)",
        dest.display(),
        proven.proof.bytes.len()
    );
    Ok(())
}

fn run_verify(path: &Path) -> Result<()> {
    if verify_bundle(&read_bundle(path)?)? {
        println!("VERIFIED");
        Ok(())
    } else {
        println!("INVALID");
        std::process::exit(1);
    }
}
