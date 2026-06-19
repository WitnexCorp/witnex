# Contributing to Witnex

Thanks for your interest in Witnex — a verifiable AI agent framework. This
document covers local setup, the pull-request process, and code style.

By participating you agree to abide by our
[Code of Conduct](CODE_OF_CONDUCT.md).

## Project principles

These come straight from the project charter and shape how we review changes:

- **Documentation rides with code.** Every PR that adds or changes a feature
  must update the relevant docs in the same PR.
- **Honest scope.** Witnex proves input/output and execution-trace integrity —
  it does **not** prove the LLM inference is correct. Don't describe the project
  in ways that overstate its guarantees.
- **Open source from day one**, dual-licensed MIT OR Apache-2.0.

## Development setup

### Prerequisites

- **Rust** ≥ 1.85 (edition 2024) — install via [rustup](https://rustup.rs/).
  The repo pins the toolchain in `rust-toolchain.toml` (`stable` + `rustfmt` +
  `clippy`).
- **Node** ≥ 20 and **pnpm** ≥ 11 — install pnpm via `corepack enable` or
  `npm i -g pnpm`.

### First build

```sh
# Rust workspace
cargo build --workspace

# TypeScript workspace
pnpm install
pnpm -r build
```

## Before you open a PR

Run the same checks CI runs (`.github/workflows/ci.yml`) locally:

```sh
# Rust
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace --all-targets
cargo test --workspace

# TypeScript
pnpm install --frozen-lockfile
pnpm lint
pnpm typecheck
```

All of these must pass. CI runs on every push to `main` and on every pull
request.

## Pull-request process

1. **Fork & branch.** Create a topic branch off `main`
   (`git switch -c feat/short-description`). Keep `main` clean.
2. **Keep PRs focused.** One logical change per PR. Smaller is easier to review.
3. **Update docs and tests** in the same PR as the code they describe.
4. **Write a clear description.** What changed, why, and how you verified it.
   Link any related issue.
5. **Green CI is required** before review. Fix `fmt`/`clippy`/`test` failures
   yourself rather than asking a reviewer to look past them.
6. **Address review feedback** by pushing follow-up commits; we squash on merge.

### Commit messages

- Use the imperative mood: "Add trace nonce", not "Added"/"Adds".
- Reference issues where relevant (`Fixes #123`).
- Group unrelated changes into separate commits/PRs.

## Code style

### Rust

- Idiomatic Rust 2024. Format with `rustfmt` (default config) — `cargo fmt`.
- `cargo clippy` must pass with `-D warnings`. No `#[allow(...)]` without a
  comment explaining why.
- Public items in library crates are documented; `witnex-core` enforces this
  with `#![deny(missing_docs)]`.
- No `unsafe` — crates set `#![forbid(unsafe_code)]`.
- Prefer explicit, descriptive names over abbreviations.

### TypeScript

- ESLint (flat config, `eslint.config.js`) and `tsc --noEmit` must both pass.
- `strict` mode is on; keep it that way. Avoid `any`.
- Keep the SDK types in sync with the canonical Rust types in `witnex-core`.

## Questions

Open a GitHub issue for bugs, feature ideas, or design discussion. For anything
security-sensitive, please follow responsible disclosure rather than filing a
public issue.
