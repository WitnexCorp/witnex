//! # witnex-core
//!
//! Core runtime types and traits for **Witnex** — a verifiable AI agent
//! framework. Every action an agent takes (LLM call, tool use, decision)
//! produces a cryptographic commitment that a third party can later verify,
//! via a ZK proof, *without re-running the execution*.
//!
//! ## Phase 1 scope
//!
//! The types in this crate model two integrity properties:
//!
//! - **Input/output integrity** — the agent commits to specific inputs and
//!   outputs and cannot tamper with them after the fact.
//! - **Execution trace integrity** — the sequence of tool calls happened in
//!   the claimed order with the claimed parameters.
//!
//! ## Phase 1 non-scope
//!
//! We do **not** prove that the LLM inference itself was correct. That is the
//! harder zkML problem, intentionally left for a later phase. A Witnex trace
//! commits to *what the model was asked and what it returned*, not to *whether
//! the returned answer is the "right" one*.
//!
//! Beyond the core types and traits, this crate provides the canonical hashing
//! and trace-commitment logic (see [`hash`] and [`trace`]). It does **not** do
//! proving or I/O — see [`witnex-prover`] and [`witnex-verifier`] for the Risc0
//! proof machinery.
//!
//! [`witnex-prover`]: https://github.com/witnex/witnex
//! [`witnex-verifier`]: https://github.com/witnex/witnex

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use core::fmt;

use serde::{Deserialize, Serialize};

pub mod hash;
pub mod llm;
pub mod trace;

/// A 32-byte SHA-256 digest.
///
/// All commitments in a [`ExecutionTrace`] are SHA-256 digests of the
/// canonical byte encoding of the committed value. Using a fixed-width newtype
/// (rather than a bare `[u8; 32]`) gives the commitments a distinct type and a
/// hex-formatted [`Debug`]/[`Display`] representation.
///
/// > Note: the derived serde representation encodes the digest as a sequence of
/// > 32 bytes. A human-friendly hex/base64 representation is a follow-up
/// > (`TODO`) for the on-disk proof format.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Digest(pub [u8; 32]);

impl Digest {
    /// The number of bytes in a SHA-256 digest.
    pub const LEN: usize = 32;
}

impl fmt::Debug for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Digest(")?;
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        write!(f, ")")
    }
}

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// Identifier of the LLM that produced an output, e.g. `"claude-opus-4-8"`.
///
/// The model id is part of the commitment: a verifier learns *which* model was
/// claimed to have run, even though Witnex does not prove the inference itself.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModelId(pub String);

/// A Unix timestamp in milliseconds.
///
/// Timestamps order events within a trace and bind the trace to a point in
/// time. They are advisory — Witnex does not prove the clock was honest.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timestamp(pub u64);

/// A per-trace random nonce.
///
/// The nonce makes each [`ExecutionTrace`] commitment unique even when the
/// input, prompt, model, and output are identical, and provides replay
/// resistance for the resulting proof.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Nonce(pub [u8; 32]);

impl fmt::Debug for Nonce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Nonce(")?;
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        write!(f, ")")
    }
}

/// A single tool invocation recorded within an [`ExecutionTrace`].
///
/// Each tool call commits to the tool's name and to digests of its input and
/// output. The position of a `ToolCall` in [`ExecutionTrace::tool_calls`] is
/// significant: execution-trace integrity means the calls occurred in exactly
/// this order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCall {
    /// The tool's name, e.g. `"web_search"`.
    pub name: String,
    /// SHA-256 digest of the canonical encoding of the tool's input arguments.
    pub input_hash: Digest,
    /// SHA-256 digest of the canonical encoding of the tool's output.
    pub output_hash: Digest,
    /// When the tool call completed.
    pub timestamp: Timestamp,
}

/// A complete, tamper-evident record of one agent execution.
///
/// An `ExecutionTrace` is the unit Witnex commits to and proves well-formed. It
/// captures, as cryptographic commitments, *what went in*, *how the model was
/// prompted*, *which model ran*, *what came out*, and *which tools were called
/// in what order*.
///
/// The trace deliberately stores **digests, not plaintext**: a verifier can
/// confirm the trace is internally consistent and matches a claimed
/// input/output without the prover having to reveal the underlying data.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionTrace {
    /// SHA-256 digest of the agent's input.
    pub input_hash: Digest,
    /// SHA-256 digest of the prompt template the agent applied to the input.
    pub prompt_template_hash: Digest,
    /// Identifier of the LLM that produced [`output_hash`](Self::output_hash).
    pub model_id: ModelId,
    /// SHA-256 digest of the agent's final output.
    pub output_hash: Digest,
    /// Tool calls made during execution, in the order they occurred.
    pub tool_calls: Vec<ToolCall>,
    /// When the execution completed.
    pub timestamp: Timestamp,
    /// Per-trace random nonce for uniqueness and replay resistance.
    pub nonce: Nonce,
}

/// An agent whose execution yields a verifiable [`ExecutionTrace`].
///
/// Implementors run some task over an input — typically calling an LLM and zero
/// or more tools — and return the trace committing to what happened. Producing
/// the trace is the implementor's responsibility; *proving* it well-formed is
/// the job of the prover crate.
///
/// The associated [`Error`](Agent::Error) type lets each agent surface its own
/// failure modes (network errors, malformed responses, …).
///
/// `execute` is `async` because a real agent performs network I/O (the LLM
/// call and any tool calls). This uses native `async fn` in traits (stable
/// since Rust 1.75); no `async_trait` crate is required. Note that the returned
/// future is not `Send`-bound here — multi-threaded executors that need `Send`
/// futures can add the bound at the call site via a where-clause.
// `async fn` in a public trait warns by default (the future has no `Send`
// bound); intentional for Phase 1, so we allow it explicitly.
#[allow(async_fn_in_trait)]
pub trait Agent {
    /// The error type returned when an execution fails before a trace is
    /// produced.
    type Error: core::error::Error + Send + Sync + 'static;

    /// Execute the agent over `input`, returning the resulting trace.
    ///
    /// No logic is implemented in this crate; this signature defines the
    /// contract every Witnex agent fulfills.
    async fn execute(&self, input: &str) -> Result<ExecutionTrace, Self::Error>;
}
