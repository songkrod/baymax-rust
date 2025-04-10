use async_trait::async_trait;
use log::{info};
use super::interface::SpeakerBackend;

pub struct GpioSpeakerBackend {
    is_active: tokio::sync::Mutex<bool>,
}

impl GpioSpeakerBackend {
    pub fn new() -> Self {
        Self {
            is_active: tokio::sync::Mutex::new(false),
        }
    }
}

#[async_trait]
impl SpeakerBackend for GpioSpeakerBackend {
    async fn play(&self, data: &[u8]) -> Result<(), String> {
        let mut active = self.is_active.lock().await;
        *active = true;

        info!("🔊 [GPIO] Playing {} bytes...", data.len());
        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

        *active = false;
        Ok(())
    }

    async fn is_busy(&self) -> bool {
        *self.is_active.lock().await
    }
}
