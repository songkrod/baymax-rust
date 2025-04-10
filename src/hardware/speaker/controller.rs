// 📁 src/hardware/speaker/controller.rs

use std::sync::Arc;
use log::info;
use async_trait::async_trait;

use super::gpio::GpioSpeakerBackend;
use super::local::OsSpeakerBackend;
use super::interface::SpeakerBackend;

pub struct SpeakerController {
    backend: Arc<dyn SpeakerBackend>,
}

impl SpeakerController {
    pub fn new(backend: Arc<dyn SpeakerBackend>) -> Arc<Self> {
        Arc::new(Self { backend })
    }
}

// ✅ ทำให้ SpeakerController เองก็เป็น SpeakerBackend ด้วย
#[async_trait]
impl SpeakerBackend for SpeakerController {
    async fn play(&self, data: &[u8]) -> Result<(), String> {
        self.backend.play(data).await
    }

    async fn is_busy(&self) -> bool {
        self.backend.is_busy().await
    }
}

// ✅ Factory function ที่เลือก backend ตาม ENV
pub fn create_speaker_controller_from_env() -> Arc<dyn SpeakerBackend> {
    let use_gpio = std::env::var("USE_GPIO").unwrap_or_else(|_| "false".to_string()) == "true";

    if use_gpio {
        info!("🔧 ใช้ GPIO speaker backend");
        SpeakerController::new(Arc::new(GpioSpeakerBackend::new()))
    } else {
        info!("🖥️ ใช้ OS/mpg123 speaker backend");
        SpeakerController::new(Arc::new(OsSpeakerBackend::new()))
    }
}