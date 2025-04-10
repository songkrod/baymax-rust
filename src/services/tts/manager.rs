// 📁 src/services/tts/manager.rs

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use log::{info, warn};
use tokio::task;
use async_trait::async_trait;

use super::interface::TTSService;
use super::google::GoogleTTS;
use super::espeak::ESpeakTTS;

pub struct SmartTTS {
    is_speaking: Arc<AtomicBool>,
    primary: Arc<dyn TTSService + Send + Sync>,
    fallback: Arc<dyn TTSService + Send + Sync>,
}

impl SmartTTS {
    pub fn new() -> Self {
        let backend = std::env::var("tts_backend").unwrap_or_else(|_| "google".to_string());

        let primary: Arc<dyn TTSService + Send + Sync> = match backend.as_str() {
            "google" => {
                info!("🗣️ Using Google TTS as primary");
                Arc::new(GoogleTTS)
            }
            "espeak" => {
                info!("🗣️ Using Local eSpeak TTS as primary");
                Arc::new(ESpeakTTS)
            }
            unknown => {
                warn!("❗ Unknown TTS_BACKEND '{}', fallback to local", unknown);
                Arc::new(ESpeakTTS)
            }
        };

        let fallback: Arc<dyn TTSService + Send + Sync> = Arc::new(ESpeakTTS);
        Self {
            is_speaking: Arc::new(AtomicBool::new(false)),
            primary,
            fallback,
        }
    }

    pub fn is_speaking(&self) -> bool {
        self.is_speaking.load(Ordering::Relaxed)
    }

    /// พูดข้อความเต็มก้อน
    pub async fn speak(&self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.is_speaking.store(true, Ordering::Relaxed);

        let result = match self.primary.speak(text).await {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("⚠️ Primary TTS failed: {}. Falling back to local.", e);
                self.fallback.speak(text).await
            }
        };

        self.is_speaking.store(false, Ordering::Relaxed);
        result
    }

    /// แยกเป็น chunk และพูดทีละก้อน (ยังไม่ปรับใช้ speaking flag ในที่นี้)
    pub async fn speak_streamed(&self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        let chunks = Self::split_into_chunks(text);

        for chunk in chunks {
            let primary = Arc::clone(&self.primary);
            let fallback = Arc::clone(&self.fallback);
            let chunk_clone = chunk.clone();

            task::spawn(async move {
                if let Err(e) = primary.speak(&chunk_clone).await {
                    warn!("⚠️ Streamed chunk failed: {}, fallback...", e);
                    let _ = fallback.speak(&chunk_clone).await;
                }
            });

            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        }

        Ok(())
    }

    fn split_into_chunks(text: &str) -> Vec<String> {
        let mut chunks = vec![];
        let mut current = String::new();

        for word in text.split_whitespace() {
            if current.len() + word.len() + 1 > 40 {
                chunks.push(current.trim().to_string());
                current.clear();
            }
            current.push_str(word);
            current.push(' ');
        }

        if !current.trim().is_empty() {
            chunks.push(current.trim().to_string());
        }

        chunks
    }
}

#[async_trait]
impl TTSService for SmartTTS {
    async fn speak(&self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.speak(text).await
    }
}
