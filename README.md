# Witnex

> Verifiable AI agent framework — every action produces a cryptographic
> commitment + ZK proof, verifiable by a third party without re-running it.

Witnex records each agent execution (LLM call, tool use, decision) as a
tamper-evident **execution trace** and proves that trace **well-formed** with a
[Risc0](https://dev.risczero.com/) ZK proof. A verifier confirms *what the agent
did* without access to the prover, the LLM, or the original plaintext.

## What Witnex proves (Phase 1)

- **Input/output integrity** — the agent committed to specific inputs and
  outputs and cannot tamper with them after the fact.
- **Execution trace integrity** — tool calls happened in the claimed order with
  the claimed parameters.

**Not in Phase 1:** Witnex does *not* prove the LLM inference itself was correct.
That is the harder zkML problem, intentionally deferred. See
[`docs/architecture.md`](docs/architecture.md).

## Repository layout

```
witnex/
├── crates/                 # Rust workspace
│   ├── witnex-core/        # Agent runtime + trace types (Agent, ExecutionTrace, ToolCall)
│   ├── witnex-prover/      # Risc0 guest + host (proof generation)
│   ├── witnex-verifier/    # Standalone verifier
│   └── witnex-cli/         # Demo CLI (`witnex`)
├── packages/               # TypeScript / pnpm workspace
│   ├── sdk/                # @witnex/sdk
│   └── examples/           # Demo apps
└── docs/                   # Architecture, positioning, ADRs
```

## Status

**Phase 1, Prompt 1 complete** — workspace scaffold, core types/traits, and the
architecture document. No proving or LLM logic is implemented yet; that is the
next slice (`witnex demo summarize` / `witnex verify`).

## Prerequisites

- **Rust** ≥ 1.85 (edition 2024) — install via [rustup](https://rustup.rs/).
  On Windows the default `msvc` target needs the Visual C++ Build Tools; the
  rustup `x86_64-pc-windows-gnu` toolchain is a self-contained alternative.
- **Node** ≥ 22.13 (required by pnpm 11.7+) and **pnpm** ≥ 11.

## Build

```sh
# Rust workspace
cargo build

# TypeScript workspace
pnpm install
pnpm -r build
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for dev setup, the PR process, and code
style. All participation is governed by our
[Code of Conduct](CODE_OF_CONDUCT.md).

## License

Dual-licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE)
- [MIT License](LICENSE-MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
