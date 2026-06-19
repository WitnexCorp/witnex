//! Witnex demo CLI.
//!
//! Phase 1 scaffolding: the command surface is sketched here but the logic is
//! implemented in Prompt 2. Planned commands:
//!
//! ```text
//! witnex demo summarize "<text>"   # run agent, emit trace + proof JSON
//! witnex verify <proof.json>       # verify a proof bundle -> VERIFIED / INVALID
//! ```

#![forbid(unsafe_code)]

fn main() {
    eprintln!("witnex: not yet implemented — see Prompt 2 (first working slice).");
    eprintln!("planned commands:");
    eprintln!("  witnex demo summarize \"<text>\"");
    eprintln!("  witnex verify <proof.json>");
    std::process::exit(1);
}
