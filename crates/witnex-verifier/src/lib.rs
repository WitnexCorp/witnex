//! # witnex-verifier
//!
//! Standalone verification of Witnex [`ProofBundle`]s.
//!
//! A verifier is the *third party* in the Witnex thesis: it confirms that a
//! proof attests to a well-formed execution trace, without access to the
//! prover, the LLM, or the original plaintext inputs and outputs.
//!
//! Phase 1 provides the [`StructuralVerifier`]: it recomputes the trace's
//! canonical commitment and compares it to the bundle's claimed commitment
//! (the public journal). This detects any post-hoc tampering of the trace, but
//! — unlike the forthcoming Risc0 path — it requires the **full trace** rather
//! than a succinct zero-knowledge proof. The Risc0 guest will prove this exact
//! commitment equality in zero knowledge; the check itself is identical.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use core::convert::Infallible;

use witnex_prover::ProofBundle;

/// The result of verifying a [`ProofBundle`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerificationOutcome {
    /// The proof is valid and the trace is well-formed.
    Verified,
    /// The proof is invalid or the trace is malformed.
    Invalid,
}

/// Verifies a bundle by recomputing the trace commitment (no ZK proof).
///
/// This checks the same property the Risc0 guest will prove — that
/// `bundle.trace`'s [`commitment`](witnex_core::ExecutionTrace::commitment)
/// equals `bundle.commitment` — but natively, requiring the full trace. It is
/// the Phase-1 stand-in for receipt verification.
#[derive(Clone, Copy, Debug, Default)]
pub struct StructuralVerifier;

impl StructuralVerifier {
    /// Recompute the trace commitment and compare it to the bundle's journal.
    ///
    /// Returns [`VerificationOutcome::Verified`] iff they match. Any tampering
    /// with the trace after commitment changes the recomputed value and yields
    /// [`VerificationOutcome::Invalid`].
    pub fn check(bundle: &ProofBundle) -> VerificationOutcome {
        if bundle.trace.commitment() == bundle.commitment {
            VerificationOutcome::Verified
        } else {
            VerificationOutcome::Invalid
        }
    }
}

impl Verifier for StructuralVerifier {
    type Error = Infallible;

    async fn verify(&self, bundle: &ProofBundle) -> Result<VerificationOutcome, Self::Error> {
        Ok(Self::check(bundle))
    }
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
