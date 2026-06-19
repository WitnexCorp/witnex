//! Building and committing to [`ExecutionTrace`]s.
//!
//! Two operations live here:
//!
//! - **Committing plaintext** into a trace — [`ExecutionTrace::commit`] and
//!   [`ToolCall::commit`] SHA-256 the raw input/output/prompt so the trace holds
//!   only digests.
//! - **The canonical commitment** — [`ExecutionTrace::commitment`] folds every
//!   committed field into a single digest using a length-prefixed encoding.
//!   This is the value a Risc0 guest re-derives to prove the trace is
//!   well-formed, and the value a verifier checks. Determinism and
//!   unambiguous field boundaries are what make that proof meaningful.

use sha2::{Digest as _, Sha256};

use crate::{Digest, ExecutionTrace, ModelId, Nonce, Timestamp, ToolCall};

/// Append `bytes` to `hasher` with a `u64` little-endian length prefix.
///
/// Length-prefixing makes the encoding injective: without it, the field pair
/// `("ab", "c")` and `("a", "bc")` would hash identically.
fn write_field(hasher: &mut Sha256, bytes: &[u8]) {
    hasher.update((bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

impl ToolCall {
    /// Commit plaintext tool-call data, SHA-256 hashing `input` and `output`.
    pub fn commit(
        name: impl Into<String>,
        input: &str,
        output: &str,
        timestamp: Timestamp,
    ) -> Self {
        Self {
            name: name.into(),
            input_hash: Digest::sha256(input),
            output_hash: Digest::sha256(output),
            timestamp,
        }
    }
}

impl ExecutionTrace {
    /// Build a trace by committing (SHA-256 hashing) the plaintext fields.
    ///
    /// `tool_calls` are committed ahead of time via [`ToolCall::commit`].
    /// `timestamp` and `nonce` are supplied by the caller (the runtime), keeping
    /// this function deterministic and free of clock/RNG access.
    pub fn commit(
        input: &str,
        prompt_template: &str,
        model_id: ModelId,
        output: &str,
        tool_calls: Vec<ToolCall>,
        timestamp: Timestamp,
        nonce: Nonce,
    ) -> Self {
        Self {
            input_hash: Digest::sha256(input),
            prompt_template_hash: Digest::sha256(prompt_template),
            model_id,
            output_hash: Digest::sha256(output),
            tool_calls,
            timestamp,
            nonce,
        }
    }

    /// The canonical commitment digest over every committed field.
    ///
    /// This is the hash chain a proof attests to. The encoding is deterministic
    /// and length-prefixed, so any change to a commitment, the model id, the
    /// tool-call sequence (including order), the timestamp, or the nonce yields
    /// a different digest.
    pub fn commitment(&self) -> Digest {
        let mut hasher = Sha256::new();

        write_field(&mut hasher, &self.input_hash.0);
        write_field(&mut hasher, &self.prompt_template_hash.0);
        write_field(&mut hasher, self.model_id.0.as_bytes());
        write_field(&mut hasher, &self.output_hash.0);

        hasher.update((self.tool_calls.len() as u64).to_le_bytes());
        for call in &self.tool_calls {
            write_field(&mut hasher, call.name.as_bytes());
            write_field(&mut hasher, &call.input_hash.0);
            write_field(&mut hasher, &call.output_hash.0);
            hasher.update(call.timestamp.0.to_le_bytes());
        }

        hasher.update(self.timestamp.0.to_le_bytes());
        write_field(&mut hasher, &self.nonce.0);

        let mut buf = [0u8; Digest::LEN];
        buf.copy_from_slice(&hasher.finalize());
        Digest(buf)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Digest, ExecutionTrace, ModelId, Nonce, Timestamp, ToolCall};

    fn sample() -> ExecutionTrace {
        ExecutionTrace::commit(
            "the input",
            "Summarize this in one sentence: {input}",
            ModelId("claude-opus-4-8".to_string()),
            "the output",
            vec![ToolCall::commit(
                "web_search",
                "query",
                "results",
                Timestamp(10),
            )],
            Timestamp(100),
            Nonce([7u8; 32]),
        )
    }

    #[test]
    fn commit_hashes_plaintext_fields() {
        let trace = sample();
        assert_eq!(trace.input_hash, Digest::sha256("the input"));
        assert_eq!(trace.output_hash, Digest::sha256("the output"));
        assert_eq!(trace.tool_calls[0].input_hash, Digest::sha256("query"));
    }

    #[test]
    fn commitment_is_deterministic() {
        assert_eq!(sample().commitment(), sample().commitment());
    }

    #[test]
    fn commitment_changes_when_output_changes() {
        let mut tampered = sample();
        tampered.output_hash = Digest::sha256("a different output");
        assert_ne!(sample().commitment(), tampered.commitment());
    }

    #[test]
    fn commitment_changes_when_nonce_changes() {
        let mut other = sample();
        other.nonce = Nonce([8u8; 32]);
        assert_ne!(sample().commitment(), other.commitment());
    }

    #[test]
    fn commitment_is_sensitive_to_tool_call_order() {
        let a = ToolCall::commit("a", "in", "out", Timestamp(1));
        let b = ToolCall::commit("b", "in", "out", Timestamp(2));

        let mut trace = sample();
        trace.tool_calls = vec![a.clone(), b.clone()];
        let forward = trace.commitment();
        trace.tool_calls = vec![b, a];
        let reversed = trace.commitment();

        assert_ne!(forward, reversed);
    }
}
