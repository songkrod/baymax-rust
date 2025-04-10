use async_trait::async_trait;

#[async_trait]
pub trait SpeakerBackend: Send + Sync {
    async fn play(&self, data: &[u8]) -> Result<(), String>;
    async fn is_busy(&self) -> bool;
}