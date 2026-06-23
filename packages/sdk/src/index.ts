/**
 * @witnex/sdk — TypeScript types for the Witnex verifiable AI agent framework.
 *
 * These interfaces mirror the canonical Rust types in `witnex-core` and
 * `witnex-prover` so that a proof bundle produced by the Rust runtime
 * round-trips through the SDK without re-shaping.
 *
 * The shapes here match the **serde-derived JSON** the Rust runtime emits:
 * - struct fields are `snake_case` (serde's default), e.g. `input_hash`,
 *   `model_id`, `tool_calls`;
 * - newtype wrappers (`Digest`, `Nonce`, `ModelId`, `Timestamp`) serialize
 *   transparently to their inner value;
 * - `Digest`, `Nonce`, and `Proof.bytes` are byte arrays (`number[]`, each
 *   element 0–255), because `[u8; 32]` / `Vec<u8>` serialize as JSON arrays.
 *
 * Phase 1: **types only**. Proving and verification are exposed in later phases.
 */

/**
 * A 32-byte SHA-256 digest.
 *
 * Serialized by the Rust runtime as an array of 32 byte values (`number[]`,
 * each 0–255), mirroring `Digest([u8; 32])` in `witnex-core`.
 *
 * > Note: a human-friendly hex/base64 on-disk encoding is a planned follow-up
 * > in `witnex-core` (see its `Digest` `TODO`). When that lands, this type
 * > becomes `string`; until then it is the raw byte array.
 */
export type Digest = number[];

/** Identifier of the LLM that produced an output, e.g. `"claude-opus-4-8"`. */
export type ModelId = string;

/** A Unix timestamp in milliseconds. */
export type Timestamp = number;

/**
 * A per-trace random nonce.
 *
 * Serialized as an array of 32 byte values (`number[]`), mirroring
 * `Nonce([u8; 32])` in `witnex-core`.
 */
export type Nonce = number[];

/** A single tool invocation recorded within an {@link ExecutionTrace}. */
export interface ToolCall {
  /** The tool's name, e.g. `"web_search"`. */
  name: string;
  /** SHA-256 digest of the canonical encoding of the tool's input arguments. */
  input_hash: Digest;
  /** SHA-256 digest of the canonical encoding of the tool's output. */
  output_hash: Digest;
  /** When the tool call completed. */
  timestamp: Timestamp;
}

/** A complete, tamper-evident record of one agent execution. */
export interface ExecutionTrace {
  /** SHA-256 digest of the agent's input. */
  input_hash: Digest;
  /** SHA-256 digest of the prompt template the agent applied to the input. */
  prompt_template_hash: Digest;
  /** Identifier of the LLM that produced {@link ExecutionTrace.output_hash}. */
  model_id: ModelId;
  /** SHA-256 digest of the agent's final output. */
  output_hash: Digest;
  /** Tool calls made during execution, in the order they occurred. */
  tool_calls: ToolCall[];
  /** When the execution completed. */
  timestamp: Timestamp;
  /** Per-trace random nonce for uniqueness and replay resistance. */
  nonce: Nonce;
}

/**
 * An opaque zero-knowledge proof that an {@link ExecutionTrace} is well-formed.
 *
 * In the Risc0 implementation this wraps a serialized receipt. In Phase 1 it is
 * a placeholder byte container that is **empty** until the guest lands.
 */
export interface Proof {
  /** Serialized Risc0 receipt as a byte array. Empty (`[]`) until implemented. */
  bytes: number[];
}

/**
 * A self-contained bundle pairing a trace with its commitment and proof.
 *
 * This is the artifact a Witnex agent emits and a verifier consumes — the
 * single JSON file produced by the demo CLI.
 */
export interface ProofBundle {
  /** The execution trace being attested to. */
  trace: ExecutionTrace;
  /**
   * The trace's canonical commitment (the public *journal*): the value the
   * proof attests to, equal to the commitment of `trace` for an untampered
   * bundle. A verifier checks `trace` against this.
   */
  commitment: Digest;
  /** The proof that `trace` is well-formed. Empty until the Risc0 guest lands. */
  proof: Proof;
}
