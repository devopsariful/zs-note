use async_trait::async_trait;
use futures::{Stream, StreamExt};
use reqwest::Client;
use serde_json::{json, Value};

use crate::ai::AIProvider;
use crate::types::*;

pub struct GeminiProvider {
    client: Client,
    api_key: String,
}

impl GeminiProvider {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("Failed to create HTTP client"),
            api_key: api_key.to_string(),
        }
    }

    fn build_contents(&self, messages: &[ChatMessage]) -> Vec<Value> {
        let mut contents = Vec::new();
        for m in messages {
            let role = match m.role {
                ChatRole::User => "user",
                ChatRole::Assistant => "model",
                ChatRole::System => "user",
                ChatRole::Tool => "user",
            };
            contents.push(json!({
                "role": role,
                "parts": [{"text": m.content}],
            }));
        }
        contents
    }
}

#[async_trait]
impl AIProvider for GeminiProvider {
    async fn chat(&self, request: ChatCompletionRequest) -> anyhow::Result<ChatCompletionResponse> {
        let contents = self.build_contents(&request.messages);
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            request.model, self.api_key
        );

        let mut payload = json!({
            "contents": contents,
        });

        let generation_config = json!({
            "maxOutputTokens": request.max_tokens.unwrap_or(4096),
            "temperature": request.temperature.unwrap_or(0.7),
        });
        payload["generationConfig"] = generation_config;

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let data: Value = response.json().await?;

        let content = data["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let usage = data.get("usageMetadata").map(|u| TokenUsage {
            prompt: u["promptTokenCount"].as_u64().unwrap_or(0) as u32,
            completion: u["candidatesTokenCount"].as_u64().unwrap_or(0) as u32,
            total: u["totalTokenCount"].as_u64().unwrap_or(0) as u32,
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
        let contents = self.build_contents(&request.messages);
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?key={}&alt=sse",
            request.model, self.api_key
        );

        let payload = json!({
            "contents": contents,
            "generationConfig": {
                "maxOutputTokens": request.max_tokens.unwrap_or(4096),
                "temperature": request.temperature.unwrap_or(0.7),
            }
        });

        let response = self
            .client
            .post(&url)
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
                                if let Some(text_content) = json_data["candidates"][0]["content"]
                                    ["parts"][0]["text"]
                                    .as_str()
                                {
                                    return StreamEvent::Token {
                                        content: text_content.to_string(),
                                    };
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

    async fn count_tokens(&self, text: &str, model: &str) -> anyhow::Result<u32> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:countTokens?key={}",
            model, self.api_key
        );

        let payload = json!({
            "contents": [{
                "parts": [{"text": text}]
            }]
        });

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let data: Value = response.json().await?;
        Ok(data["totalTokens"].as_u64().unwrap_or(0) as u32)
    }

    async fn list_models(&self) -> anyhow::Result<Vec<String>> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models?key={}",
            self.api_key
        );

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
