// src/services/tts/manager.rs

use std::sync::Arc;
use log::{info, warn};

use super::interface::TTSService;
use super::google_custom::GoogleTTS as GoogleCustomTTS;
use super::local::LocalTTS;

pub struct SmartTTS {
    primary: Arc<dyn TTSService + Send + Sync>,
    fallback: Arc<dyn TTSService + Send + Sync>,
}

impl SmartTTS {
    /// สร้างระบบ TTS ที่สามารถสลับ backend ได้ผ่าน ENV (pluggable)
    pub fn new() -> Self {
        let backend = std::env::var("tts_backend").unwrap_or_else(|_| "google_custom".to_string());

        let primary: Arc<dyn TTSService + Send + Sync> = match backend.as_str() {
            "google_custom" => {
                info!("🗣️ Using Google TTS (Custom Auth) as primary");
                Arc::new(GoogleCustomTTS)
            }
            "local" => {
                info!("🗣️ Using Local eSpeak/say TTS as primary");
                Arc::new(LocalTTS)
            }
            unknown => {
                warn!("❗ Unknown TTS_BACKEND '{}', fallback to local", unknown);
                Arc::new(LocalTTS)
            }
        };

        let fallback: Arc<dyn TTSService + Send + Sync> = Arc::new(LocalTTS);
        Self { primary, fallback }
    }

    /// สั่งให้พูดด้วย primary และ fallback อัตโนมัติหาก primary ล้มเหลว
    pub async fn speak(&self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        match self.primary.speak(text).await {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("⚠️ Primary TTS failed: {}. Falling back to local.", e);
                self.fallback.speak(text).await
            }
        }
    }
}