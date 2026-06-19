# Witnex Architecture

> Status: Prompt 1 (scaffold) and Prompt 2 (working slice) are implemented. The
> Risc0 guest builds and proves in **dev mode**; a full real-STARK,
> receipt-checked end-to-end path is the remaining work (see Status).

Witnex turns each agent execution into a **tamper-evident commitment** and a
**ZK proof** that the commitment is well-formed, so a third party can verify
*what the agent did* without re-running it.

## What is proven (and what is not)

| Property | Proven? |
|----------|---------|
| The agent committed to a specific input and output (input/output integrity) | ✅ |
| Tool calls occurred in the claimed order with claimed parameters (trace integrity) | ✅ |
| The committed inputs/outputs/tool data form a consistent hash chain | ✅ |
| The LLM inference itself was *correct* | ❌ (zkML — later phase) |

A Witnex proof says *"this exact input, prompted this exact way, run against this
named model, produced this exact output, via this exact sequence of tool calls."*
It does **not** say the output is the right answer.

## Components

| Component | Crate / package | Role |
|-----------|-----------------|------|
| Runtime + commitment types | `witnex-core` | `Agent`, `ExecutionTrace`, `ToolCall`, `Digest`; `ExecutionTrace::commit`/`commitment` (the canonical SHA-256 hash chain); the `LlmBackend` trait + `MockBackend`. |
| LLM backend | `witnex-anthropic` | `AnthropicBackend` — `LlmBackend` over the Claude Messages API. |
| Proof bundle types | `witnex-prover` | `ProofBundle { trace, commitment, proof }`. |
| Structural verifier | `witnex-verifier` | `StructuralVerifier` — recomputes the commitment (Phase-1, needs the full trace). |
| Demo CLI | `witnex-cli` | `witnex demo summarize` (mock or Anthropic, env-gated) / `witnex verify` (structural). |
| zkVM (separate workspace) | `zkvm/` | Risc0 guest (recompute commitment → journal), host `prove`/`verify`, and the `witnex-zkvm` CLI (`prove`/`verify` with a real receipt). |
| SDK | `@witnex/sdk` | TypeScript mirror of the trace/proof types. |

## Two verification modes

| Mode | Where | Checks | Needs |
|------|-------|--------|-------|
| **Structural** (Phase 1) | `witnex verify` (`witnex-verifier`) | recompute `trace.commitment()` == `bundle.commitment` | the full trace; no toolchain |
| **ZK receipt** | `witnex-zkvm verify` (`zkvm/host`) | Risc0 receipt verifies against the guest image id; journal binds to the trace | the rzup toolchain (Linux/macOS/WSL) |

Both check the *same property* — that the commitment was correctly derived. The
ZK receipt additionally proves it cryptographically (the right guest ran), and
is the path toward succinct verification without re-execution.

## End-to-end sequence

```mermaid
sequenceDiagram
    autonumber
    actor Caller as Caller / Developer
    participant Agent as Agent (witnex-core)
    participant LLM as Backend (Anthropic / Mock)
    participant Prover as zkVM host + guest (zkvm/)
    participant Verifier as Verifier (third party)

    Note over Caller,Agent: 1) Agent invocation
    Caller->>Agent: summarize(input)
    Agent->>Agent: input_hash, prompt_template_hash = SHA256(...)

    Note over Agent,LLM: 2) Trace generation
    Agent->>LLM: complete(model, system, messages, max_tokens)
    LLM-->>Agent: output
    Agent->>Agent: output_hash; record tool_calls, timestamp, nonce
    Agent->>Agent: commitment = SHA256 hash chain over all fields
    Agent-->>Caller: ProofBundle { trace, commitment, proof:empty }

    Note over Caller,Prover: 3) Proof generation (witnex-zkvm prove)
    Caller->>Prover: prove(trace)
    Prover->>Prover: guest recomputes the commitment, commits it to the journal
    Prover-->>Caller: ProofBundle with embedded Risc0 receipt

    Note over Caller,Verifier: 4) Verification (no re-execution)
    Caller->>Verifier: bundle.json
    alt structural (witnex verify)
        Verifier->>Verifier: recompute commitment == bundle.commitment
    else ZK receipt (witnex-zkvm verify)
        Verifier->>Verifier: receipt.verify(image id); journal == commitment
    end
    Verifier-->>Caller: VERIFIED / INVALID
```

## Trust model

- The ZK verifier trusts the **Risc0 proof system** and the published **guest
  image id**, nothing else about the prover.
- The verifier does **not** trust the agent's host, network, or LLM provider.
- Timestamps and the model id are *committed*, not *proven honest* — a dishonest
  agent can still claim a wrong clock or a wrong model; what it cannot do is
  alter the input/output/tool-call commitments after the fact.

## Status & next steps

- [x] **Prompt 1** — workspace scaffold, core types, this document.
- [x] **Prompt 2** — `witnex demo summarize` / `witnex verify`, canonical trace
      commitment, mocked + real (Anthropic) backends, and an end-to-end
      integration test with a mocked LLM.
- [x] **Risc0 guest** — recomputes the commitment in the zkVM; host
      `prove`/`verify`; `witnex-zkvm` CLI; receipt embedded in `ProofBundle`.
      Built and dev-mode-tested (CI + a Linux box).
- [ ] **Real STARK end-to-end** — a full non-dev proof run, and wiring real
      receipt verification into the main `witnex verify` (currently structural).
- [ ] **zkML** — proving the inference itself (out of Phase 1 scope).
