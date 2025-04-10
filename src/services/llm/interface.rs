// 📁 src/services/llm/interface.rs

use async_trait::async_trait;

#[async_trait]
pub trait LLMService: Send + Sync {
    async fn chat(&self, input: &str) -> Result<String, Box<dyn std::error::Error>>;
}

#[async_trait]
pub trait LLMStreamable: Send + Sync {
    async fn stream_chat<F>(&self, input: &str, on_chunk: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut(String) + Send + 'static;
}
