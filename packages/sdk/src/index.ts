/**
 * @witnex/sdk — TypeScript types for the Witnex verifiable AI agent framework.
 *
 * These interfaces mirror the canonical Rust types in `witnex-core` so that a
 * proof bundle produced by the Rust runtime round-trips through the SDK.
 *
 * Phase 1: **types only**. Proving and verification are exposed in later phases.
 */

/** A 32-byte SHA-256 digest, encoded as a lowercase hex string. */
export type Digest = string;

/** Identifier of the LLM that produced an output, e.g. `"claude-opus-4-8"`. */
export type ModelId = string;

/** A Unix timestamp in milliseconds. */
export type Timestamp = number;

/** A per-trace random nonce, encoded as a lowercase hex string. */
export type Nonce = string;

/** A single tool invocation recorded within an {@link ExecutionTrace}. */
export interface ToolCall {
  /** The tool's name, e.g. `"web_search"`. */
  name: string;
  /** SHA-256 digest of the canonical encoding of the tool's input arguments. */
  inputHash: Digest;
  /** SHA-256 digest of the canonical encoding of the tool's output. */
  outputHash: Digest;
  /** When the tool call completed. */
  timestamp: Timestamp;
}

/** A complete, tamper-evident record of one agent execution. */
export interface ExecutionTrace {
  /** SHA-256 digest of the agent's input. */
  inputHash: Digest;
  /** SHA-256 digest of the prompt template the agent applied to the input. */
  promptTemplateHash: Digest;
  /** Identifier of the LLM that produced {@link outputHash}. */
  modelId: ModelId;
  /** SHA-256 digest of the agent's final output. */
  outputHash: Digest;
  /** Tool calls made during execution, in the order they occurred. */
  toolCalls: ToolCall[];
  /** When the execution completed. */
  timestamp: Timestamp;
  /** Per-trace random nonce for uniqueness and replay resistance. */
  nonce: Nonce;
}

/** An opaque zero-knowledge proof that an {@link ExecutionTrace} is well-formed. */
export interface Proof {
  /** Serialized Risc0 receipt, base64-encoded. */
  bytes: string;
}

/** A self-contained bundle pairing a trace with its proof. */
export interface ProofBundle {
  trace: ExecutionTrace;
  proof: Proof;
}
