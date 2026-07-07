use async_trait::async_trait;
use futures::{Stream, StreamExt};
use reqwest::Client;
use serde_json::{json, Value};

use crate::ai::AIProvider;
use crate::types::*;

pub struct ClaudeProvider {
    client: Client,
    api_key: String,
}

impl ClaudeProvider {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("Failed to create HTTP client"),
            api_key: api_key.to_string(),
        }
    }

    fn build_messages(&self, messages: &[ChatMessage]) -> Vec<Value> {
        messages
            .iter()
            .map(|m| {
                json!({
                    "role": match m.role {
                        ChatRole::User => "user",
                        ChatRole::Assistant => "assistant",
                        ChatRole::System => "system",
                        ChatRole::Tool => "user",
                    },
                    "content": m.content,
                })
            })
            .collect()
    }
}

#[async_trait]
impl AIProvider for ClaudeProvider {
    async fn chat(&self, request: ChatCompletionRequest) -> anyhow::Result<ChatCompletionResponse> {
        let messages = self.build_messages(&request.messages);

        let mut payload = json!({
            "model": request.model,
            "messages": messages,
            "max_tokens": request.max_tokens.unwrap_or(4096),
        });

        if let Some(temp) = request.temperature {
            payload["temperature"] = json!(temp);
        }

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let data: Value = response.json().await?;

        let content = data["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let usage = data.get("usage").map(|u| TokenUsage {
            prompt: u["input_tokens"].as_u64().unwrap_or(0) as u32,
            completion: u["output_tokens"].as_u64().unwrap_or(0) as u32,
            total: (u["input_tokens"].as_u64().unwrap_or(0)
                + u["output_tokens"].as_u64().unwrap_or(0)) as u32,
        });

        Ok(ChatCompletionResponse {
            message: ChatMessage {
                role: ChatRole::Assistant,
                content,
                timestamp: chrono::Utc::now(),
                model: Some(request.model),
            },
            usage,
        })
    }

    async fn stream_chat(
        &self,
        request: ChatCompletionRequest,
    ) -> anyhow::Result<Box<dyn Stream<Item = StreamEvent> + Send + Unpin>> {
        let messages = self.build_messages(&request.messages);

        let mut payload = json!({
            "model": request.model,
            "messages": messages,
            "max_tokens": request.max_tokens.unwrap_or(4096),
            "stream": true,
        });

        if let Some(temp) = request.temperature {
            payload["temperature"] = json!(temp);
        }

        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let stream = response
            .bytes_stream()
            .map(|chunk_result| match chunk_result {
                Ok(chunk) => {
                    let text = String::from_utf8_lossy(&chunk);
                    for line in text.lines() {
                        if let Some(data) = line.strip_prefix("data: ") {
                            if let Ok(json_data) = serde_json::from_str::<Value>(data) {
                                if json_data["type"] == "content_block_delta" {
                                    if let Some(text_content) = json_data["delta"]["text"].as_str()
                                    {
                                        return StreamEvent::Token {
                                            content: text_content.to_string(),
                                        };
                                    }
                                }
                                if json_data["type"] == "message_stop" {
                                    return StreamEvent::Done { usage: None };
                                }
                            }
                        }
                    }
                    StreamEvent::Token {
                        content: String::new(),
                    }
                }
                Err(e) => StreamEvent::Error {
                    message: e.to_string(),
                },
            });

        Ok(Box::new(stream))
    }

    async fn count_tokens(&self, text: &str, _model: &str) -> anyhow::Result<u32> {
        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages/count_tokens")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&json!({
                "model": "claude-3-haiku-20240307",
                "messages": [{"role": "user", "content": text}],
            }))
            .send()
            .await?;

        let data: Value = response.json().await?;
        Ok(data["input_tokens"].as_u64().unwrap_or(0) as u32)
    }

    async fn list_models(&self) -> anyhow::Result<Vec<String>> {
        let response = self
            .client
            .get("https://api.anthropic.com/v1/models")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .send()
            .await?;

        let data: Value = response.json().await?;
        let models = data["data"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| m["id"].as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(models)
    }

    async fn test_connection(&self) -> anyhow::Result<bool> {
        self.list_models().await?;
        Ok(true)
    }
}
