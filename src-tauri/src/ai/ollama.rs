use async_trait::async_trait;
use futures::{Stream, StreamExt};
use reqwest::Client;
use serde_json::{json, Value};

use crate::ai::AIProvider;
use crate::types::*;

pub struct OllamaProvider {
    client: Client,
    base_url: String,
}

impl OllamaProvider {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(300))
                .build()
                .expect("Failed to create HTTP client"),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }
}

#[async_trait]
impl AIProvider for OllamaProvider {
    async fn chat(&self, request: ChatCompletionRequest) -> anyhow::Result<ChatCompletionResponse> {
        let messages: Vec<Value> = request
            .messages
            .iter()
            .map(|m| {
                json!({
                    "role": m.role.to_string(),
                    "content": m.content,
                })
            })
            .collect();

        let payload = json!({
            "model": request.model,
            "messages": messages,
            "stream": false,
            "options": {
                "temperature": request.temperature.unwrap_or(0.7),
                "num_predict": request.max_tokens.unwrap_or(4096),
            }
        });

        let url = format!("{}/api/chat", self.base_url);
        let response = self.client.post(&url).json(&payload).send().await?;
        let data: Value = response.json().await?;

        let content = data["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(ChatCompletionResponse {
            message: ChatMessage {
                role: ChatRole::Assistant,
                content,
                timestamp: chrono::Utc::now(),
                model: Some(request.model),
            },
            usage: None,
        })
    }

    async fn stream_chat(
        &self,
        request: ChatCompletionRequest,
    ) -> anyhow::Result<Box<dyn Stream<Item = StreamEvent> + Send + Unpin>> {
        let messages: Vec<Value> = request
            .messages
            .iter()
            .map(|m| {
                json!({
                    "role": m.role.to_string(),
                    "content": m.content,
                })
            })
            .collect();

        let payload = json!({
            "model": request.model,
            "messages": messages,
            "stream": true,
            "options": {
                "temperature": request.temperature.unwrap_or(0.7),
                "num_predict": request.max_tokens.unwrap_or(4096),
            }
        });

        let url = format!("{}/api/chat", self.base_url);
        let response = self.client.post(&url).json(&payload).send().await?;

        let stream = response
            .bytes_stream()
            .map(|chunk_result| match chunk_result {
                Ok(chunk) => {
                    let text = String::from_utf8_lossy(&chunk);
                    for line in text.lines() {
                        if line.is_empty() {
                            continue;
                        }
                        if let Ok(json_data) = serde_json::from_str::<Value>(line) {
                            if json_data
                                .get("done")
                                .and_then(|d| d.as_bool())
                                .unwrap_or(false)
                            {
                                let usage = json_data
                                    .get("eval_count")
                                    .and_then(|c| c.as_u64())
                                    .map(|c| TokenUsage {
                                        prompt: json_data
                                            .get("prompt_eval_count")
                                            .and_then(|v| v.as_u64())
                                            .unwrap_or(0)
                                            as u32,
                                        completion: c as u32,
                                        total: (c + json_data
                                            .get("prompt_eval_count")
                                            .and_then(|v| v.as_u64())
                                            .unwrap_or(0))
                                            as u32,
                                    });
                                return StreamEvent::Done { usage };
                            }
                            if let Some(content) = json_data["message"]["content"].as_str() {
                                return StreamEvent::Token {
                                    content: content.to_string(),
                                };
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

    async fn count_tokens(&self, text: &str, model: &str) -> anyhow::Result<u32> {
        let payload = json!({
            "model": model,
            "prompt": text,
        });

        let url = format!("{}/api/tokenize", self.base_url);
        let response = self.client.post(&url).json(&payload).send().await?;
        let data: Value = response.json().await?;

        Ok(data["tokens"]
            .as_array()
            .map(|t| t.len() as u32)
            .unwrap_or(0))
    }

    async fn list_models(&self) -> anyhow::Result<Vec<String>> {
        let url = format!("{}/api/tags", self.base_url);
        let response = self.client.get(&url).send().await?;
        let data: Value = response.json().await?;

        let models = data["models"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
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
