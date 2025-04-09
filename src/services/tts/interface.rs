use async_trait::async_trait;

#[async_trait]
pub trait TTSService: Send + Sync {
    async fn speak(&self, text: &str) -> Result<(), Box<dyn std::error::Error>>;
}