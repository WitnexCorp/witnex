//! Witnex Risc0 guest.
//!
//! Proves that an [`ExecutionTrace`] is **well-formed**: it reads the trace,
//! recomputes the canonical commitment (the SHA-256 hash chain), and commits
//! that digest to the journal. A verifier checks the receipt against the guest
//! image id and reads the committed commitment — without re-running this.
//!
//! Phase 1 non-scope: this does NOT prove the LLM inference was correct.

use risc0_zkvm::guest::env;
use witnex_core::{Digest, ExecutionTrace};

fn main() {
    // Private input: the full execution trace.
    let trace: ExecutionTrace = env::read();

    // Recompute the commitment inside the zkVM — same logic as the host/CLI.
    let commitment: Digest = trace.commitment();

    // Public output (journal): the verified commitment.
    env::commit(&commitment);
}
