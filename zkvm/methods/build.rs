// Compiles the guest crate(s) listed in Cargo.toml `[package.metadata.risc0]`
// and writes `methods.rs` (the ELF bytes + image id) into OUT_DIR.
fn main() {
    risc0_build::embed_methods();
}
