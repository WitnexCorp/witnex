//! Witnex Risc0 host — prove and verify execution-trace well-formedness.
//!
//! [`prove`] runs the guest over a trace and returns a [`Receipt`] whose public
//! journal is the trace's canonical commitment. [`verify`] checks a receipt
//! against the guest image id and returns that committed commitment.
//!
//! Set `RISC0_DEV_MODE=1` to execute the guest and produce a *dev* receipt
//! quickly (no real STARK) — useful for tests and CI.

use anyhow::{Context, Result};
use risc0_zkvm::{ExecutorEnv, Receipt, default_prover};
use witnex_core::{Digest, ExecutionTrace};
use witnex_methods::{WITNEX_GUEST_ELF, WITNEX_GUEST_ID};

/// Prove that `trace` is well-formed.
///
/// The returned receipt's journal is the trace's
/// [`commitment`](ExecutionTrace::commitment), recomputed inside the zkVM.
pub fn prove(trace: &ExecutionTrace) -> Result<Receipt> {
    let env = ExecutorEnv::builder()
        .write(trace)
        .context("writing trace to guest env")?
        .build()
        .context("building executor env")?;

    let receipt = default_prover()
        .prove(env, WITNEX_GUEST_ELF)
        .context("proving guest execution")?
        .receipt;

    Ok(receipt)
}

/// Verify a receipt against the guest image id and return the committed
/// commitment from its journal.
pub fn verify(receipt: &Receipt) -> Result<Digest> {
    receipt
        .verify(WITNEX_GUEST_ID)
        .context("verifying receipt against guest image id")?;
    let commitment: Digest = receipt.journal.decode().context("decoding journal")?;
    Ok(commitment)
}

#[cfg(test)]
mod tests {
    use super::*;
    use witnex_core::{ModelId, Nonce, Timestamp, ToolCall};

    fn sample_trace() -> ExecutionTrace {
        ExecutionTrace::commit(
            "input text",
            "Summarize this in one sentence: {input}",
            ModelId("claude-opus-4-8".to_string()),
            "the one-sentence summary",
            vec![ToolCall::commit("web_search", "query", "results", Timestamp(1))],
            Timestamp(1_700_000_000_000),
            Nonce([9u8; 32]),
        )
    }

    /// Run with `RISC0_DEV_MODE=1` for a fast dev receipt.
    #[test]
    fn prove_then_verify_recovers_commitment() {
        let trace = sample_trace();
        let receipt = prove(&trace).expect("proving should succeed");
        let journal = verify(&receipt).expect("verifying should succeed");
        // The guest recomputed the exact same commitment as the host.
        assert_eq!(journal, trace.commitment());
    }
}
