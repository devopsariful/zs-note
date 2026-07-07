pub mod claude;
pub mod gemini;
pub mod ollama;
pub mod openai;

use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};

pub use crate::types::{ChatCompletionRequest, ChatCompletionResponse, StreamEvent};

#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn chat(&self, request: ChatCompletionRequest) -> anyhow::Result<ChatCompletionResponse>;
    async fn stream_chat(
        &self,
        request: ChatCompletionRequest,
    ) -> anyhow::Result<Box<dyn Stream<Item = StreamEvent> + Send + Unpin>>;
    async fn count_tokens(&self, text: &str, model: &str) -> anyhow::Result<u32>;
    async fn list_models(&self) -> anyhow::Result<Vec<String>>;
    async fn test_connection(&self) -> anyhow::Result<bool>;
}

pub struct ProviderRouter {
    providers: Vec<(String, Box<dyn AIProvider + Send + Sync>)>,
    default_provider: String,
}

impl ProviderRouter {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
            default_provider: "ollama".into(),
        }
    }

    pub fn register(&mut self, name: &str, provider: Box<dyn AIProvider + Send + Sync>) {
        self.providers.push((name.to_string(), provider));
    }

    pub fn get(&self, name: &str) -> Option<&(dyn AIProvider + Send + Sync)> {
        self.providers
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, p)| p.as_ref())
    }

    pub fn default(&self) -> &(dyn AIProvider + Send + Sync) {
        self.get(&self.default_provider)
            .or_else(|| self.providers.first().map(|(_, p)| p.as_ref()))
            .expect("No AI providers registered")
    }
}

impl Default for ProviderRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextInjectorConfig {
    pub include_document: bool,
    pub include_selection: bool,
    pub include_rag: bool,
    pub max_context_length: usize,
}

impl Default for ContextInjectorConfig {
    fn default() -> Self {
        Self {
            include_document: true,
            include_selection: true,
            include_rag: true,
            max_context_length: 8192,
        }
    }
}

pub struct ContextInjector {
    config: ContextInjectorConfig,
}

impl ContextInjector {
    pub fn new(config: ContextInjectorConfig) -> Self {
        Self { config }
    }

    pub fn inject_context(
        &self,
        messages: &mut Vec<crate::types::ChatMessage>,
        document_content: Option<&str>,
        selection: Option<&str>,
        rag_results: Option<&[crate::types::SearchResult]>,
    ) {
        let mut context_parts: Vec<String> = Vec::new();

        if let Some(doc) = document_content {
            if self.config.include_document {
                context_parts.push(format!("Current document content:\n{}", doc));
            }
        }

        if let Some(sel) = selection {
            if self.config.include_selection {
                context_parts.push(format!("Selected text:\n{}", sel));
            }
        }

        if let Some(results) = rag_results {
            if self.config.include_rag && !results.is_empty() {
                let rag_context: Vec<String> = results
                    .iter()
                    .map(|r| format!("- From {}: {}", r.title, r.snippet))
                    .collect();
                context_parts.push(format!("Relevant context:\n{}", rag_context.join("\n")));
            }
        }

        if !context_parts.is_empty() {
            let context_string = context_parts.join("\n\n");
            let truncated = if context_string.len() > self.config.max_context_length {
                format!(
                    "{}...[truncated]",
                    &context_string[..self.config.max_context_length]
                )
            } else {
                context_string
            };

            messages.insert(
                0,
                crate::types::ChatMessage {
                    role: crate::types::ChatRole::System,
                    content: truncated,
                    timestamp: chrono::Utc::now(),
                    model: None,
                },
            );
        }
    }
}
