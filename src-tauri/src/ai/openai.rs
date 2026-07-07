use async_trait::async_trait;
use futures::{Stream, StreamExt};
use reqwest::Client;
use serde_json::{json, Value};

use crate::ai::AIProvider;
use crate::types::*;

pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    base_url: String,
}

impl OpenAIProvider {
    pub fn new(api_key: &str, base_url: Option<&str>) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("Failed to create HTTP client"),
            api_key: api_key.to_string(),
            base_url: base_url
                .unwrap_or("https://api.openai.com/v1")
                .trim_end_matches('/')
                .to_string(),
        }
    }

    fn build_chat_payload(&self, request: ChatCompletionRequest) -> Value {
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

        let mut payload = json!({
            "model": request.model,
            "messages": messages,
            "stream": request.stream,
        });

        if let Some(temp) = request.temperature {
            payload["temperature"] = json!(temp);
        }
        if let Some(max_tokens) = request.max_tokens {
            payload["max_tokens"] = json!(max_tokens);
        }
        if let Some(top_p) = request.top_p {
            payload["top_p"] = json!(top_p);
        }

        payload
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    async fn chat(&self, request: ChatCompletionRequest) -> anyhow::Result<ChatCompletionResponse> {
        let payload = self.build_chat_payload(request);
        let url = format!("{}/chat/completions", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let data: Value = response.json().await?;

        let content = data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let usage = data.get("usage").map(|u| TokenUsage {
            prompt: u["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion: u["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total: u["total_tokens"].as_u64().unwrap_or(0) as u32,
        });

        Ok(ChatCompletionResponse {
            message: ChatMessage {
                role: ChatRole::Assistant,
                content,
                timestamp: chrono::Utc::now(),
                model: Some(data["model"].as_str().unwrap_or("").to_string()),
            },
            usage,
        })
    }

    async fn stream_chat(
        &self,
        request: ChatCompletionRequest,
    ) -> anyhow::Result<Box<dyn Stream<Item = StreamEvent> + Send + Unpin>> {
        let payload = self.build_chat_payload(request);
        let url = format!("{}/chat/completions", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
            .json(&payload)
            .send()
            .await?;

        let stream = response
            .bytes_stream()
            .map(|chunk_result| match chunk_result {
                Ok(chunk) => {
                    let text = String::from_utf8_lossy(&chunk);
                    let mut last_event = StreamEvent::Token {
                        content: String::new(),
                    };

                    for line in text.lines() {
                        if let Some(data) = line.strip_prefix("data: ") {
                            if data == "[DONE]" {
                                return StreamEvent::Done { usage: None };
                            }
                            if let Ok(json_data) = serde_json::from_str::<Value>(data) {
                                if let Some(delta) =
                                    json_data["choices"][0]["delta"]["content"].as_str()
                                {
                                    last_event = StreamEvent::Token {
                                        content: delta.to_string(),
                                    };
                                }
                            }
                        }
                    }
                    last_event
                }
                Err(e) => StreamEvent::Error {
                    message: e.to_string(),
                },
            });

        Ok(Box::new(stream))
    }

    async fn count_tokens(&self, text: &str, model: &str) -> anyhow::Result<u32> {
        let url = format!("{}/chat/completions", self.base_url);
        let payload = json!({
            "model": model,
            "messages": [{"role": "user", "content": text}],
            "max_tokens": 1,
            "stream": false,
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let data: Value = response.json().await?;
        Ok(data["usage"]["prompt_tokens"].as_u64().unwrap_or(0) as u32)
    }

    async fn list_models(&self) -> anyhow::Result<Vec<String>> {
        let url = format!("{}/models", self.base_url);
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
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
