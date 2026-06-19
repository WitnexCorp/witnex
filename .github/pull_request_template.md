## Summary

<!-- What does this PR change, and why? Keep it focused on one logical change. -->

## Related issues

<!-- e.g. Fixes #123 -->

## Verification

<!-- How did you test this? Paste relevant command output. -->

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`
- [ ] `pnpm lint` / `pnpm typecheck` (if TS changed)

## Checklist

- [ ] Docs updated in the same PR as the code they describe
- [ ] No overstated guarantees (Witnex proves trace integrity, not LLM correctness)
- [ ] One logical change per PR
