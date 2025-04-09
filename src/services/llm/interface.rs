use async_trait::async_trait;

#[async_trait]
pub trait LLMService: Send + Sync {
    async fn chat(&self, input: &str) -> Result<String, Box<dyn std::error::Error>>;
}