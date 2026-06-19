# Security Policy

Witnex is a verifiable-computation project: the integrity of its commitments and
proofs is the whole point. We take security reports seriously.

## Supported versions

Witnex is pre-1.0 and under active development. Only the latest `main` is
supported. Pin a commit if you need stability.

## Reporting a vulnerability

**Please do not open a public issue for security vulnerabilities.**

Report privately via GitHub Security Advisories:

> [Report a vulnerability](https://github.com/WitnexCorp/witnex/security/advisories/new)

Include, where possible:

- A description of the issue and its impact.
- Steps to reproduce or a proof of concept.
- Affected component (`witnex-core`, `witnex-prover`, `witnex-verifier`,
  `witnex-cli`, or `@witnex/sdk`) and commit hash.

We aim to acknowledge reports within a few business days and will keep you
updated as we investigate. Please give us a reasonable window to fix the issue
before any public disclosure.

## Scope notes

Witnex Phase 1 proves **execution-trace integrity**, not LLM-inference
correctness. Reports that a model produced a "wrong" answer are out of scope —
but anything that lets a trace or proof misrepresent what was committed (hash
collisions in our usage, malleable commitments, verifier bypasses, etc.) is
firmly in scope.
