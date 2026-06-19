//! # witnex-prover
//!
//! Proof generation for Witnex execution traces, backed by the [Risc0] zkVM.
//!
//! In Phase 1 the guest program proves only that an [`ExecutionTrace`] is
//! **well-formed** — that its commitments form a consistent hash chain — *not*
//! that the LLM inference was correct.
//!
//! This module currently defines **only types** (no proving logic). The Risc0
//! host/guest integration lands in Prompt 2.
//!
//! [Risc0]: https://dev.risczero.com/

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use serde::{Deserialize, Serialize};
use witnex_core::ExecutionTrace;

/// An opaque zero-knowledge proof that an [`ExecutionTrace`] is well-formed.
///
/// In the Risc0 implementation this wraps a serialized receipt (the STARK proof
/// plus its public journal). For now it is a placeholder byte container.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proof {
    /// Serialized proof bytes (Risc0 receipt). Empty until implemented.
    pub bytes: Vec<u8>,
}

/// A self-contained bundle pairing a trace with its proof.
///
/// This is the artifact a Witnex agent emits and a verifier consumes — the
/// single JSON file produced by the demo CLI.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProofBundle {
    /// The execution trace being attested to.
    pub trace: ExecutionTrace,
    /// The proof that `trace` is well-formed.
    pub proof: Proof,
}

/// Produces [`Proof`]s for execution traces.
///
/// Implementors run the Risc0 guest over a trace and return the resulting
/// proof. No implementation is provided in Phase 1 scaffolding.
///
/// `prove` is `async` because proof generation is typically offloaded
/// (spawned on a blocking pool, or sent to a remote proving service). Uses
/// native `async fn` in traits — no `async_trait` crate.
#[allow(async_fn_in_trait)]
pub trait Prover {
    /// The error type returned when proving fails.
    type Error: core::error::Error + Send + Sync + 'static;

    /// Generate a proof that `trace` is well-formed.
    async fn prove(&self, trace: &ExecutionTrace) -> Result<Proof, Self::Error>;
}
