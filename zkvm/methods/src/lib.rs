//! Generated guest method constants.
//!
//! `risc0-build` writes `methods.rs` into `OUT_DIR` during the build; it defines
//! `WITNEX_GUEST_ELF: &[u8]` (the compiled guest) and `WITNEX_GUEST_ID: [u32; 8]`
//! (its image id), named after the guest binary `witnex-guest`.
include!(concat!(env!("OUT_DIR"), "/methods.rs"));
