# Witnex zkVM (Risc0)

The zero-knowledge layer: a [Risc0](https://dev.risczero.com/) guest that proves
a Witnex `ExecutionTrace` is **well-formed** — that recomputing its canonical
commitment (the SHA-256 hash chain) inside the zkVM yields the value published
in the receipt journal. Phase 1 non-scope: this does **not** prove the LLM
inference was correct.

This is a **separate workspace**, excluded from the root `Cargo.toml`, because
building the guest needs the Risc0 RISC-V toolchain — available on **Linux,
macOS, or WSL**, not native Windows.

## Layout

```
zkvm/
├── methods/            # build glue: compiles the guest, embeds ELF + image id
│   ├── build.rs        # risc0_build::embed_methods()
│   └── guest/          # the guest program (runs in the zkVM)
│       └── src/main.rs # read trace -> recompute commitment -> commit to journal
└── host/               # prove() / verify() over the guest
    └── src/lib.rs
```

The guest reuses `witnex-core::ExecutionTrace::commitment` so the proof attests
to the exact same hash chain the CLI and structural verifier compute.

## Prerequisites

```sh
# Rust (host) + the Risc0 toolchain (guest)
curl -L https://risczero.com/install | bash
rzup install
```

## Build & test

```sh
# Fast: execute the guest and produce a dev receipt (no real STARK).
RISC0_DEV_MODE=1 cargo test

# Real proof (slow, CPU/RAM heavy — needs a capable machine).
cargo test --release
```

`RISC0_DEV_MODE=1` is what CI (`.github/workflows/zkvm.yml`) and low-resource
machines should use; it validates the guest logic and the prove/verify flow
without the cost of STARK generation.

## Status

Scaffold toward the real ZK proof. Next: wire the serialized receipt into
`witnex_prover::ProofBundle.proof.bytes` and have `witnex verify` check the
receipt (replacing the Phase-1 structural recompute).
