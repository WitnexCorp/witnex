//! Anthropic Claude backend for the Witnex [`LlmBackend`] trait.
//!
//! Calls the [Messages API] (`POST /v1/messages`). [`LlmRequest`] already
//! mirrors the API request shape (model, top-level `system`, `max_tokens`,
//! `messages`), so it serializes directly as the request body.
//!
//! [Messages API]: https://docs.claude.com

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use serde::Deserialize;
use witnex_core::ModelId;
use witnex_core::llm::{LlmBackend, LlmRequest, LlmResponse};

const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Errors returned by [`AnthropicBackend`].
#[derive(Debug, thiserror::Error)]
pub enum AnthropicError {
    /// `ANTHROPIC_API_KEY` is not set in the environment.
    #[error("ANTHROPIC_API_KEY is not set")]
    MissingApiKey,
    /// Transport / HTTP-client error.
    #[error("request failed: {0}")]
    Http(#[from] reqwest::Error),
    /// The API responded with a non-success status.
    #[error("anthropic API error (status {status}): {body}")]
    Api {
        /// HTTP status code.
        status: u16,
        /// Raw response body.
        body: String,
    },
}

/// An [`LlmBackend`] backed by the Anthropic Messages API.
#[derive(Clone)]
pub struct AnthropicBackend {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl AnthropicBackend {
    /// Construct from an explicit API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            base_url: DEFAULT_BASE_URL.to_string(),
        }
    }

    /// Construct from the `ANTHROPIC_API_KEY` environment variable.
    pub fn from_env() -> Result<Self, AnthropicError> {
        let key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| AnthropicError::MissingApiKey)?;
        Ok(Self::new(key))
    }

    /// Override the API base URL (e.g. for a proxy or a test server).
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }
}

/// Subset of the Messages API response we consume.
#[derive(Deserialize)]
struct ApiResponse {
    model: String,
    content: Vec<ContentBlock>,
    stop_reason: Option<String>,
}

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    text: String,
}

impl LlmBackend for AnthropicBackend {
    type Error = AnthropicError;

    async fn complete(&self, request: &LlmRequest) -> Result<LlmResponse, Self::Error> {
        let url = format!("{}/v1/messages", self.base_url);
        let resp = self
            .client
            .post(url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .json(request) // LlmRequest mirrors the Messages API body
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(AnthropicError::Api {
                status: status.as_u16(),
                body,
            });
        }

        let api: ApiResponse = resp.json().await?;
        // Concatenate the text blocks (Phase 1 ignores non-text content).
        let content: String = api
            .content
            .iter()
            .filter(|b| b.kind == "text")
            .map(|b| b.text.as_str())
            .collect();

        Ok(LlmResponse {
            model: ModelId(api.model),
            content,
            stop_reason: api.stop_reason,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_serializes_to_messages_api_body() {
        let req = LlmRequest::single_turn("Be concise.", "Hello", 256);
        let v: serde_json::Value = serde_json::to_value(&req).unwrap();
        assert_eq!(v["model"], "claude-opus-4-8");
        assert_eq!(v["max_tokens"], 256);
        assert_eq!(v["system"], "Be concise.");
        assert_eq!(v["messages"][0]["role"], "user");
        assert_eq!(v["messages"][0]["content"], "Hello");
    }

    #[test]
    fn new_uses_default_base_url() {
        let b = AnthropicBackend::new("test-key");
        assert_eq!(b.base_url, DEFAULT_BASE_URL);
    }
}
