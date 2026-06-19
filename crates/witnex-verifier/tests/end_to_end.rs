//! End-to-end integration test for the Witnex Phase-1 pipeline.
//!
//! Exercises the full loop with a **mocked LLM** (no network): fill the
//! summarize template -> call the backend -> commit an `ExecutionTrace` ->
//! assemble a `ProofBundle` -> round-trip through JSON (as the CLI does via a
//! file) -> verify structurally. Mirrors `witnex demo summarize` + `witnex
//! verify` without spawning the binary.

use witnex_core::llm::{LlmBackend, LlmRequest, MockBackend};
use witnex_core::{Digest, ExecutionTrace, Nonce, Timestamp};
use witnex_prover::{Proof, ProofBundle};
use witnex_verifier::{StructuralVerifier, VerificationOutcome};

const TEMPLATE: &str = "Summarize this in one sentence: {input}";

/// Run the agent over `input` with a mocked backend and build a proof bundle —
/// the same sequence the CLI's `summarize` performs.
fn run_agent(input: &str) -> ProofBundle {
    let backend = MockBackend::new("a mocked one-sentence summary.");
    let filled = TEMPLATE.replace("{input}", input);
    let request = LlmRequest::single_turn("Be concise.", filled, 256);
    let response = pollster::block_on(backend.complete(&request)).expect("mock is infallible");

    let trace = ExecutionTrace::commit(
        input,
        TEMPLATE,
        response.model.clone(),
        &response.content,
        Vec::new(),
        // Fixed timestamp/nonce so the test is deterministic.
        Timestamp(1_700_000_000_000),
        Nonce([42u8; 32]),
    );
    let commitment = trace.commitment();
    ProofBundle {
        trace,
        commitment,
        proof: Proof { bytes: Vec::new() },
    }
}

#[test]
fn full_loop_with_json_roundtrip_verifies() {
    let bundle = run_agent("the quick brown fox jumps over the lazy dog");

    // The agent committed to the mocked output.
    assert_eq!(
        bundle.trace.output_hash,
        Digest::sha256("a mocked one-sentence summary.")
    );

    // Serialize and re-parse, as `verify` reads from a file.
    let json = serde_json::to_string(&bundle).expect("serialize");
    let parsed: ProofBundle = serde_json::from_str(&json).expect("parse");
    assert_eq!(parsed, bundle);

    assert_eq!(
        StructuralVerifier::check(&parsed),
        VerificationOutcome::Verified
    );
}

#[test]
fn tampered_trace_is_rejected() {
    let mut bundle = run_agent("the quick brown fox");
    // Forge the output after committing — the recomputed commitment no longer
    // matches the journal.
    bundle.trace.output_hash = Digest::sha256("a forged output");
    assert_eq!(
        StructuralVerifier::check(&bundle),
        VerificationOutcome::Invalid
    );
}

#[test]
fn tampered_journal_is_rejected() {
    let mut bundle = run_agent("hello world");
    bundle.commitment = Digest::sha256("not the real commitment");
    assert_eq!(
        StructuralVerifier::check(&bundle),
        VerificationOutcome::Invalid
    );
}

#[test]
fn distinct_inputs_produce_distinct_commitments() {
    let a = run_agent("input one");
    let b = run_agent("input two");
    assert_ne!(a.commitment, b.commitment);
}
