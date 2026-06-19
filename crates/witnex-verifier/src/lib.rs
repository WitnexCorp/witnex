//! # witnex-verifier
//!
//! Standalone verification of Witnex [`ProofBundle`]s.
//!
//! A verifier is the *third party* in the Witnex thesis: it confirms that a
//! proof attests to a well-formed execution trace, without access to the
//! prover, the LLM, or the original plaintext inputs and outputs.
//!
//! This module currently defines **only types** (no verification logic). The
//! Risc0 receipt verification lands in Prompt 2.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use witnex_prover::ProofBundle;

/// The result of verifying a [`ProofBundle`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerificationOutcome {
    /// The proof is valid and the trace is well-formed.
    Verified,
    /// The proof is invalid or the trace is malformed.
    Invalid,
}

/// Verifies Witnex proof bundles.
///
/// Implementors check a bundle's proof against the expected guest image id and
/// public journal. No implementation is provided in Phase 1 scaffolding.
///
/// `verify` is `async` to accommodate verification backends that perform I/O
/// (e.g. fetching a guest image id from a registry or a remote verifier
/// service). Uses native `async fn` in traits — no `async_trait` crate.
#[allow(async_fn_in_trait)]
pub trait Verifier {
    /// The error type returned when verification cannot be completed (distinct
    /// from a well-formed proof that is simply [`Invalid`]).
    ///
    /// [`Invalid`]: VerificationOutcome::Invalid
    type Error: core::error::Error + Send + Sync + 'static;

    /// Verify `bundle`, returning whether it is valid.
    async fn verify(&self, bundle: &ProofBundle) -> Result<VerificationOutcome, Self::Error>;
}
