/**
 * Example: round-trip a Witnex `ProofBundle` through the SDK types.
 *
 * The Rust demo CLI emits a bundle with `witnex demo summarize "<text>"`. This
 * example parses that exact JSON shape into the SDK's {@link ProofBundle} type
 * and performs a **structural** sanity check: every digest is a 32-byte array,
 * tool calls are well-formed, and the commitment is present.
 *
 * It does NOT recompute the commitment or verify the ZK proof — recomputation
 * lives in the Rust verifier today, and proof verification is a later phase. The
 * point here is to show the SDK types faithfully model what the runtime emits.
 *
 * Run: `pnpm --filter @witnex/examples build && node dist/round-trip.js`
 */

import type { Digest, ProofBundle, ToolCall } from "@witnex/sdk";

/**
 * A sample bundle in the exact serde-derived JSON shape the Rust CLI writes:
 * `snake_case` fields, `Digest`/`Nonce`/`proof.bytes` as byte arrays, a
 * top-level `commitment`, and (in Phase 1) an empty `proof.bytes`.
 *
 * The digest/nonce/commitment bytes below are illustrative placeholders — a
 * real bundle's digests are SHA-256 outputs over the canonical encodings.
 */
const SAMPLE_BUNDLE_JSON = `{
  "trace": {
    "input_hash": [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31],
    "prompt_template_hash": [31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0],
    "model_id": "claude-opus-4-8",
    "output_hash": [9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9],
    "tool_calls": [],
    "timestamp": 1750700000000,
    "nonce": [255, 254, 253, 252, 251, 250, 249, 248, 247, 246, 245, 244, 243, 242, 241, 240, 239, 238, 237, 236, 235, 234, 233, 232, 231, 230, 229, 228, 227, 226, 225, 224]
  },
  "commitment": [42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42],
  "proof": { "bytes": [] }
}`;

/** SHA-256 produces 32-byte digests; {@link Digest} mirrors that fixed width. */
const DIGEST_LEN = 32;

/** Narrow an arbitrary value to a 32-byte digest array. */
function isDigest(value: unknown): value is Digest {
  return (
    Array.isArray(value) &&
    value.length === DIGEST_LEN &&
    value.every((b) => Number.isInteger(b) && b >= 0 && b <= 255)
  );
}

function assert(condition: boolean, message: string): asserts condition {
  if (!condition) {
    throw new Error(`bundle is malformed: ${message}`);
  }
}

/**
 * Parse and structurally validate a Witnex proof bundle.
 *
 * Throws if the JSON does not match the {@link ProofBundle} shape. Returns the
 * typed bundle on success.
 */
export function parseBundle(json: string): ProofBundle {
  const bundle = JSON.parse(json) as ProofBundle;

  assert(isDigest(bundle.commitment), "commitment must be a 32-byte digest");

  const { trace } = bundle;
  assert(isDigest(trace.input_hash), "trace.input_hash must be a 32-byte digest");
  assert(
    isDigest(trace.prompt_template_hash),
    "trace.prompt_template_hash must be a 32-byte digest",
  );
  assert(isDigest(trace.output_hash), "trace.output_hash must be a 32-byte digest");
  assert(isDigest(trace.nonce), "trace.nonce must be a 32-byte array");
  assert(typeof trace.model_id === "string", "trace.model_id must be a string");
  assert(typeof trace.timestamp === "number", "trace.timestamp must be a number");
  assert(Array.isArray(trace.tool_calls), "trace.tool_calls must be an array");

  trace.tool_calls.forEach((call: ToolCall, i) => {
    assert(typeof call.name === "string", `tool_calls[${i}].name must be a string`);
    assert(isDigest(call.input_hash), `tool_calls[${i}].input_hash must be a 32-byte digest`);
    assert(isDigest(call.output_hash), `tool_calls[${i}].output_hash must be a 32-byte digest`);
  });

  assert(Array.isArray(bundle.proof.bytes), "proof.bytes must be a byte array");

  return bundle;
}

/** Render a digest as a lowercase hex string for display. */
function toHex(digest: Digest): string {
  return digest.map((b) => b.toString(16).padStart(2, "0")).join("");
}

function main(): void {
  const bundle = parseBundle(SAMPLE_BUNDLE_JSON);

  console.log("Parsed a structurally valid Witnex proof bundle:");
  console.log(`  model:      ${bundle.trace.model_id}`);
  console.log(`  tool calls: ${bundle.trace.tool_calls.length}`);
  console.log(`  commitment: ${toHex(bundle.commitment)}`);
  console.log(
    `  proof:      ${bundle.proof.bytes.length} bytes` +
      (bundle.proof.bytes.length === 0 ? " (placeholder — Risc0 guest not yet wired)" : ""),
  );
}

main();
