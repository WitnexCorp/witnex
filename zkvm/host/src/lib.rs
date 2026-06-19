//! Witnex Risc0 host — prove and verify execution-trace well-formedness.
//!
//! Two layers:
//!
//! - Low level: [`prove`] runs the guest over a trace and returns a [`Receipt`]
//!   whose journal is the trace's canonical commitment; [`verify`] checks a
//!   receipt against the guest image id and returns that commitment.
//! - Bundle level: [`attach_proof`] proves a [`ProofBundle`]'s trace and embeds
//!   the serialized receipt; [`verify_bundle`] checks the embedded receipt
//!   cryptographically and binds it to the bundle.
//!
//! Set `RISC0_DEV_MODE=1` to produce a *dev* receipt quickly (no real STARK).

use anyhow::{Context, Result, bail};
use risc0_zkvm::{ExecutorEnv, Receipt, default_prover};
use witnex_core::{Digest, ExecutionTrace};
use witnex_methods::{WITNEX_GUEST_ELF, WITNEX_GUEST_ID};
use witnex_prover::ProofBundle;

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

/// Prove `bundle.trace` and return a new bundle carrying the serialized receipt
/// in `proof.bytes` and the journal commitment.
pub fn attach_proof(bundle: &ProofBundle) -> Result<ProofBundle> {
    let receipt = prove(&bundle.trace)?;
    let commitment = verify(&receipt)?;
    let bytes = bincode::serialize(&receipt).context("serializing receipt")?;
    Ok(ProofBundle {
        trace: bundle.trace.clone(),
        commitment,
        proof: witnex_prover::Proof { bytes },
    })
}

/// Verify a bundle's embedded Risc0 receipt and bind it to the bundle.
///
/// Returns `true` iff: the receipt verifies against the guest image id, its
/// journal equals `bundle.commitment`, and that commitment equals
/// `bundle.trace.commitment()` (binding the proof to this exact trace).
pub fn verify_bundle(bundle: &ProofBundle) -> Result<bool> {
    if bundle.proof.bytes.is_empty() {
        bail!("bundle has no proof (empty proof.bytes) — was it proven with witnex-zkvm?");
    }
    let receipt: Receipt =
        bincode::deserialize(&bundle.proof.bytes).context("deserializing receipt")?;
    let journal = verify(&receipt)?;
    Ok(journal == bundle.commitment && bundle.trace.commitment() == journal)
}

#[cfg(test)]
mod tests {
    use super::*;
    use witnex_core::{ModelId, Nonce, Timestamp, ToolCall};
    use witnex_prover::Proof;

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

    fn sample_bundle() -> ProofBundle {
        let trace = sample_trace();
        let commitment = trace.commitment();
        ProofBundle { trace, commitment, proof: Proof { bytes: Vec::new() } }
    }

    /// Run with `RISC0_DEV_MODE=1` for a fast dev receipt.
    #[test]
    fn prove_then_verify_recovers_commitment() {
        let trace = sample_trace();
        let receipt = prove(&trace).expect("proving should succeed");
        let journal = verify(&receipt).expect("verifying should succeed");
        assert_eq!(journal, trace.commitment());
    }

    #[test]
    fn bundle_roundtrips_through_json_and_verifies() {
        let proven = attach_proof(&sample_bundle()).expect("attach_proof");
        assert!(!proven.proof.bytes.is_empty());

        // Round-trip through JSON, as the witnex-zkvm CLI does via a file.
        let json = serde_json::to_string(&proven).expect("serialize");
        let parsed: ProofBundle = serde_json::from_str(&json).expect("parse");

        assert!(verify_bundle(&parsed).expect("verify_bundle"));
    }

    #[test]
    fn tampered_trace_fails_bundle_verification() {
        let mut proven = attach_proof(&sample_bundle()).expect("attach_proof");
        // Forge the output after proving: the proof's journal no longer matches
        // the recomputed trace commitment.
        proven.trace.output_hash = Digest::sha256("a forged output");
        assert!(!verify_bundle(&proven).expect("verify_bundle"));
    }
}
