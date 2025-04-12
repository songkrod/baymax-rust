// 📁 src/services/llm/manager.rs

use super::interface::{LLMService, LLMStreamable};
use super::open_ai::OpenAIGPT;
use std::sync::Arc;
use log::{info, warn};

pub enum LLMBackend {
    OpenAI {
        client: Arc<dyn LLMService>,
        stream: Arc<OpenAIGPT>,
    },
}

pub struct SmartLLM {
    backend: LLMBackend,
}

impl SmartLLM {
    pub fn new() -> Self {
        let backend = std::env::var("llm_backend").unwrap_or_else(|_| "openai".to_string());

        let backend = match backend.as_str() {
            "openai" => {
                info!("🤖 Using OpenAI GPT as LLM backend");
                let openai = Arc::new(OpenAIGPT);
                LLMBackend::OpenAI {
                    client: openai.clone(),
                    stream: openai,
                }
            }
            unknown => {
                warn!("❗ Unknown LLM_BACKEND '{}', fallback to OpenAI", unknown);
                let openai = Arc::new(OpenAIGPT);
                LLMBackend::OpenAI {
                    client: openai.clone(),
                    stream: openai,
                }
            }
        };

        Self { backend }
    }

    pub async fn complete(&self, input: &str) -> Result<String, Box<dyn std::error::Error>> {
        match &self.backend {
            LLMBackend::OpenAI { client, .. } => client.chat(input).await,
        }
    }

    pub async fn stream_reply<F>(&self, input: &str, mut on_chunk: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut(String) + Send + 'static,
    {
        match &self.backend {
            LLMBackend::OpenAI { stream, .. } => {
                LLMStreamable::stream_chat(stream.as_ref(), input, on_chunk).await
            }
        }
    }
}
