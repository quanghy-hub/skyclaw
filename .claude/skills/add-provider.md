# Skill: Add a new AI provider to SkyClaw

## When to use

Use this skill when the user asks to add a new AI/LLM provider (e.g., Google Gemini, Mistral, Cohere, AWS Bedrock, a custom local model server) to SkyClaw.

## Reference implementation

Study the existing providers:
- `crates/skyclaw-providers/src/anthropic.rs` -- full Provider implementation with SSE streaming, tool use, error handling
- `crates/skyclaw-providers/src/openai_compat.rs` -- OpenAI-compatible provider (works with many backends)
- `crates/skyclaw-core/src/traits/provider.rs` -- the `Provider` trait definition

## Steps

### 1. Create the provider source file

Create `crates/skyclaw-providers/src/<provider_name>.rs` using the template below.

### 2. Add the module to lib.rs

Edit `crates/skyclaw-providers/src/lib.rs`:
- Add `pub mod <provider_name>;`
- Add `pub use <provider_name>::<ProviderName>Provider;`
- Add a match arm in `create_provider()` for the new provider name

### 3. Add dependencies if needed

Edit `crates/skyclaw-providers/Cargo.toml`:
- Add any provider-specific dependencies (most providers just use `reqwest` which is already included)

### 4. Write tests

Include tests in the provider source file:
- Test `name()` returns the correct string
- Test `build_request_body()` produces valid JSON
- Test SSE event parsing if the provider uses streaming
- Test error handling for various HTTP status codes
- Test `with_base_url()` builder pattern

### 5. Verify

```bash
cargo check -p skyclaw-providers
cargo test -p skyclaw-providers
cargo clippy -p skyclaw-providers -- -D warnings
```

## Template

```rust
//! <ProviderName> provider -- <brief description>.

use async_trait::async_trait;
use futures::stream::BoxStream;
use reqwest::Client;
use serde::Deserialize;
use skyclaw_core::types::error::SkyclawError;
use skyclaw_core::types::message::{
    ChatMessage, CompletionRequest, CompletionResponse, ContentPart, MessageContent, Role,
    StreamChunk, ToolDefinition, Usage,
};
use skyclaw_core::Provider;
use tracing::{debug, error};

/// <ProviderName> API provider.
pub struct <ProviderName>Provider {
    client: Client,
    api_key: String,
    base_url: String,
}

impl <ProviderName>Provider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.<provider>.com".to_string(),
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    /// Build the JSON body for the <ProviderName> API.
    fn build_request_body(
        &self,
        request: &CompletionRequest,
        stream: bool,
    ) -> Result<serde_json::Value, SkyclawError> {
        // TODO: Convert CompletionRequest to provider-specific format
        // - Map Role::User, Role::Assistant, Role::System, Role::Tool
        // - Convert ToolDefinition to provider's tool format
        // - Handle system prompt placement (inline vs. separate field)

        let messages: Vec<serde_json::Value> = request
            .messages
            .iter()
            .map(|m| convert_message(m))
            .collect::<Result<Vec<_>, _>>()?;

        let mut body = serde_json::json!({
            "model": request.model,
            "messages": messages,
        });

        if let Some(max_tokens) = request.max_tokens {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }

        if let Some(temp) = request.temperature {
            body["temperature"] = serde_json::json!(temp);
        }

        if !request.tools.is_empty() {
            let tools: Vec<serde_json::Value> = request
                .tools
                .iter()
                .map(convert_tool)
                .collect();
            body["tools"] = serde_json::json!(tools);
        }

        if stream {
            body["stream"] = serde_json::json!(true);
        }

        Ok(body)
    }
}

// ---------------------------------------------------------------------------
// Provider-specific API response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct ApiResponse {
    // TODO: Define provider-specific response fields
    id: String,
    // choices, content, usage, etc.
}

// ---------------------------------------------------------------------------
// Conversion helpers
// ---------------------------------------------------------------------------

fn convert_message(msg: &ChatMessage) -> Result<serde_json::Value, SkyclawError> {
    let role = match msg.role {
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::System => "system",
        Role::Tool => "tool",
    };

    let content = match &msg.content {
        MessageContent::Text(text) => serde_json::json!(text),
        MessageContent::Parts(parts) => {
            // TODO: Convert content parts to provider format
            let blocks: Vec<serde_json::Value> = parts
                .iter()
                .map(|p| match p {
                    ContentPart::Text { text } => serde_json::json!({
                        "type": "text",
                        "text": text,
                    }),
                    ContentPart::ToolUse { id, name, input } => serde_json::json!({
                        "type": "tool_use",
                        "id": id,
                        "name": name,
                        "input": input,
                    }),
                    ContentPart::ToolResult { tool_use_id, content, is_error } => serde_json::json!({
                        "type": "tool_result",
                        "tool_use_id": tool_use_id,
                        "content": content,
                        "is_error": is_error,
                    }),
                })
                .collect();
            serde_json::json!(blocks)
        }
    };

    Ok(serde_json::json!({
        "role": role,
        "content": content,
    }))
}

fn convert_tool(tool: &ToolDefinition) -> serde_json::Value {
    // TODO: Adapt to provider's tool/function calling format
    serde_json::json!({
        "name": tool.name,
        "description": tool.description,
        "parameters": tool.parameters,
    })
}

// ---------------------------------------------------------------------------
// Provider trait implementation
// ---------------------------------------------------------------------------

#[async_trait]
impl Provider for <ProviderName>Provider {
    fn name(&self) -> &str {
        "<provider_name>"
    }

    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, SkyclawError> {
        let body = self.build_request_body(&request, false)?;

        debug!(provider = "<provider_name>", model = %request.model, "Sending completion request");

        let response = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| SkyclawError::Provider(format!("Request failed: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".into());
            error!(provider = "<provider_name>", %status, "API error: {}", error_body);
            if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                return Err(SkyclawError::RateLimited(error_body));
            }
            if status == reqwest::StatusCode::UNAUTHORIZED {
                return Err(SkyclawError::Auth(error_body));
            }
            return Err(SkyclawError::Provider(format!(
                "API error ({status}): {error_body}"
            )));
        }

        // TODO: Parse provider-specific response into CompletionResponse
        todo!("Parse response into CompletionResponse")
    }

    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<BoxStream<'_, Result<StreamChunk, SkyclawError>>, SkyclawError> {
        let body = self.build_request_body(&request, true)?;

        debug!(provider = "<provider_name>", model = %request.model, "Sending streaming request");

        let response = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| SkyclawError::Provider(format!("Stream request failed: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "unknown error".into());
            if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                return Err(SkyclawError::RateLimited(error_body));
            }
            if status == reqwest::StatusCode::UNAUTHORIZED {
                return Err(SkyclawError::Auth(error_body));
            }
            return Err(SkyclawError::Provider(format!(
                "API error ({status}): {error_body}"
            )));
        }

        // SSE streaming pattern -- parse server-sent events from the byte stream
        let byte_stream = response.bytes_stream();

        let event_stream = futures::stream::unfold(
            (byte_stream, String::new()),
            |(mut byte_stream, mut buffer)| async move {
                use futures::StreamExt;

                loop {
                    // Try to extract a complete SSE event from the buffer
                    if let Some(pos) = buffer.find("\n\n") {
                        let event_text: String = buffer.drain(..=pos + 1).collect();

                        // Parse "data: " lines
                        for line in event_text.lines() {
                            if let Some(data) = line.strip_prefix("data: ") {
                                if data == "[DONE]" {
                                    return None;
                                }
                                // TODO: Parse the SSE data JSON into a StreamChunk
                                // Return text deltas, tool use blocks, and stop reasons
                                // Example:
                                // let chunk = StreamChunk {
                                //     delta: Some(parsed_text),
                                //     tool_use: None,
                                //     stop_reason: None,
                                // };
                                // return Some((Ok(chunk), (byte_stream, buffer)));
                            }
                        }
                        continue;
                    }

                    // Need more data from the stream
                    match byte_stream.next().await {
                        Some(Ok(bytes)) => {
                            let text = String::from_utf8_lossy(&bytes);
                            buffer.push_str(&text);
                        }
                        Some(Err(e)) => {
                            return Some((
                                Err(SkyclawError::Provider(format!("Stream read error: {e}"))),
                                (byte_stream, buffer),
                            ));
                        }
                        None => return None,
                    }
                }
            },
        );

        Ok(Box::pin(event_stream))
    }

    async fn health_check(&self) -> Result<bool, SkyclawError> {
        // TODO: Implement a lightweight check (HEAD request or models list)
        let resp = self
            .client
            .get(format!("{}/v1/models", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| SkyclawError::Provider(format!("Health check failed: {e}")))?;

        Ok(resp.status().is_success())
    }

    async fn list_models(&self) -> Result<Vec<String>, SkyclawError> {
        // TODO: Query the provider's models endpoint, or return a static list
        Ok(vec![
            // "model-name-here".to_string(),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_name() {
        let provider = <ProviderName>Provider::new("key".to_string());
        assert_eq!(provider.name(), "<provider_name>");
    }

    #[test]
    fn with_base_url() {
        let provider = <ProviderName>Provider::new("key".to_string())
            .with_base_url("https://custom.api.com".to_string());
        assert_eq!(provider.base_url, "https://custom.api.com");
    }

    #[test]
    fn build_request_body_basic() {
        let provider = <ProviderName>Provider::new("key".to_string());
        let request = CompletionRequest {
            model: "test-model".to_string(),
            messages: vec![ChatMessage {
                role: Role::User,
                content: MessageContent::Text("Hello".to_string()),
            }],
            tools: Vec::new(),
            max_tokens: Some(1024),
            temperature: Some(0.7),
            system: None,
        };

        let body = provider.build_request_body(&request, false).unwrap();
        assert_eq!(body["model"], "test-model");
    }

    #[test]
    fn build_request_body_with_stream_flag() {
        let provider = <ProviderName>Provider::new("key".to_string());
        let request = CompletionRequest {
            model: "m".to_string(),
            messages: vec![ChatMessage {
                role: Role::User,
                content: MessageContent::Text("Hi".to_string()),
            }],
            tools: Vec::new(),
            max_tokens: None,
            temperature: None,
            system: None,
        };

        let body = provider.build_request_body(&request, true).unwrap();
        assert_eq!(body["stream"], true);
    }
}
```

## Key conventions

- **Error mapping**: Map HTTP 429 to `SkyclawError::RateLimited`, 401 to `SkyclawError::Auth`, everything else to `SkyclawError::Provider`.
- **SSE streaming**: Use `futures::stream::unfold` over the `response.bytes_stream()` to parse server-sent events. Buffer incomplete lines. Handle `[DONE]` or provider-specific end markers.
- **Tool use**: Map SkyClaw's `ToolDefinition` to the provider's function/tool calling format. Accumulate partial JSON for streaming tool calls.
- **Builder pattern**: Always provide `new(api_key)` and `with_base_url(url)` constructors.
- **Health check**: Implement a lightweight check that verifies API reachability without consuming tokens.
- **System prompt**: Handle system prompt placement according to the provider's API (separate field vs. first message).
- **No cross-impl deps**: Providers must not depend on each other. Shared utilities go in `skyclaw-core`.
