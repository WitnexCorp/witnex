//! Pluggable LLM backend abstraction.
//!
//! Witnex is provider-agnostic; the first backend is the Anthropic Claude
//! [Messages API]. The types here mirror that request/response shape so a real
//! backend is a thin mapping onto the wire format, while tests run against the
//! in-process [`MockBackend`] with no network access.
//!
//! Phase 1 keeps message content **text-only**. The real API supports
//! structured content blocks (images, tool use, …); modelling those is left for
//! a later phase.
//!
//! [Messages API]: https://docs.claude.com

use core::convert::Infallible;

use serde::{Deserialize, Serialize};

use crate::ModelId;

/// Default Claude model id used when a request does not specify one.
///
/// Per Anthropic guidance this tracks the latest, most capable Claude model.
pub const DEFAULT_MODEL: &str = "claude-opus-4-8";

/// The author of a [`Message`].
///
/// Mirrors the Anthropic Messages API roles. The `system` prompt is **not** a
/// role here — it is a top-level field on [`LlmRequest`], matching the API.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// A message from the user / caller.
    User,
    /// A message produced by the assistant (the model).
    Assistant,
}

/// A single turn in a conversation.
///
/// Content is plain text in Phase 1. Conversations must start with a
/// [`Role::User`] message and alternate roles, as the Messages API requires.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    /// Who authored this message.
    pub role: Role,
    /// The message text.
    pub content: String,
}

impl Message {
    /// Construct a [`Role::User`] message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
        }
    }

    /// Construct a [`Role::Assistant`] message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
        }
    }
}

/// A request to an [`LlmBackend`], mirroring the Anthropic Messages API.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmRequest {
    /// Model id, e.g. [`DEFAULT_MODEL`].
    pub model: ModelId,
    /// Optional system prompt (top-level, not a message — as in the API).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// The conversation so far. Must be non-empty and start with a user turn.
    pub messages: Vec<Message>,
    /// Maximum number of tokens to generate. Required by the API.
    pub max_tokens: u32,
}

impl LlmRequest {
    /// Build a single-turn request from a system prompt and one user message,
    /// using [`DEFAULT_MODEL`].
    pub fn single_turn(
        system: impl Into<String>,
        user: impl Into<String>,
        max_tokens: u32,
    ) -> Self {
        Self {
            model: ModelId(DEFAULT_MODEL.to_string()),
            system: Some(system.into()),
            messages: vec![Message::user(user)],
            max_tokens,
        }
    }
}

/// A response from an [`LlmBackend`].
///
/// Carries the final text plus the metadata Witnex commits to in a trace: the
/// model that actually answered and why generation stopped.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmResponse {
    /// The model that produced this response.
    pub model: ModelId,
    /// The generated text (the concatenation of the response's text blocks).
    pub content: String,
    /// Why generation stopped, e.g. `"end_turn"` or `"max_tokens"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
}

/// A pluggable large-language-model backend.
///
/// The real implementation calls a provider (Anthropic first); tests use
/// [`MockBackend`]. Uses native `async fn` in traits — no `async_trait` crate.
#[allow(async_fn_in_trait)]
pub trait LlmBackend {
    /// The error type returned when a completion fails.
    type Error: core::error::Error + Send + Sync + 'static;

    /// Run a completion for `request`, returning the model's response.
    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, Self::Error>;
}

/// An in-process [`LlmBackend`] that returns a fixed response without any I/O.
///
/// Used to drive deterministic, offline tests of the Witnex pipeline.
#[derive(Clone, Debug)]
pub struct MockBackend {
    /// The text every completion returns.
    response_text: String,
    /// The model id reported on the response.
    model: ModelId,
}

impl MockBackend {
    /// Create a mock backend that always returns `response_text`, reporting
    /// [`DEFAULT_MODEL`] as the responding model.
    pub fn new(response_text: impl Into<String>) -> Self {
        Self {
            response_text: response_text.into(),
            model: ModelId(DEFAULT_MODEL.to_string()),
        }
    }
}

impl LlmBackend for MockBackend {
    type Error = Infallible;

    async fn complete(&self, _request: &LlmRequest) -> Result<LlmResponse, Self::Error> {
        Ok(LlmResponse {
            model: self.model.clone(),
            content: self.response_text.clone(),
            stop_reason: Some("end_turn".to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_turn_uses_default_model_and_one_user_message() {
        let req = LlmRequest::single_turn("Be terse.", "Hello", 256);
        assert_eq!(req.model, ModelId(DEFAULT_MODEL.to_string()));
        assert_eq!(req.system.as_deref(), Some("Be terse."));
        assert_eq!(req.messages, vec![Message::user("Hello")]);
        assert_eq!(req.max_tokens, 256);
    }

    #[test]
    fn mock_backend_returns_fixed_text_offline() {
        let backend = MockBackend::new("a one sentence summary.");
        let req = LlmRequest::single_turn("Summarize.", "long input text", 64);
        let resp = pollster::block_on(backend.complete(&req)).expect("mock is infallible");
        assert_eq!(resp.content, "a one sentence summary.");
        assert_eq!(resp.model, ModelId(DEFAULT_MODEL.to_string()));
        assert_eq!(resp.stop_reason.as_deref(), Some("end_turn"));
    }
}
