use super::interface::LLMService;
use super::open_ai::OpenAIGPT;
use std::sync::Arc;
use log::{info, warn};

pub struct SmartLLM {
    primary: Arc<dyn LLMService + Send + Sync>,
}

impl SmartLLM {
    pub fn new() -> Self {
        let backend = std::env::var("llm_backend").unwrap_or_else(|_| "openai".to_string());

        let primary: Arc<dyn LLMService + Send + Sync> = match backend.as_str() {
            "openai" => {
                info!("🤖 Using OpenAI GPT as LLM backend");
                Arc::new(OpenAIGPT)
            }
            unknown => {
                warn!("❗ Unknown LLM_BACKEND '{}', fallback to OpenAI", unknown);
                Arc::new(OpenAIGPT)
            }
        };

        Self { primary }
    }

    pub async fn complete(&self, input: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.primary.chat(input).await
    }
}