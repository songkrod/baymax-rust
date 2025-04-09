use std::sync::Arc;
use log::{info, warn};
use tokio::task;
use super::interface::TTSService;
use super::google::GoogleTTS;
use super::espeak::ESpeakTTS;

pub struct SmartTTS {
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
        Self { primary, fallback }
    }

    pub async fn speak(&self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match self.primary.speak(text).await {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("⚠️ Primary TTS failed: {}. Falling back to local.", e);
                self.fallback.speak(text).await
            }
        }
    }

    /// 🎧 พูดแบบสตรีม — ตัดเป็นคำหรือประโยคย่อย แล้วพูดทีละอัน
    pub async fn speak_streamed(&self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        let chunks = Self::split_into_chunks(text);

        for chunk in chunks {
            let primary = Arc::clone(&self.primary);
            let fallback = Arc::clone(&self.fallback);
            let chunk_clone = chunk.clone();

            task::spawn(async move {
                if let Err(e) = primary.speak(&chunk_clone).await {
                    warn!("⚠️ Streamed chunk failed: {}, fallback...", e);
                    let _ = fallback.speak(&chunk_clone)
                        .await
                        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e });
                }
            });
            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        }

        Ok(())
    }

    /// 🔤 แยกข้อความเป็นช่วง ๆ (เช่น ประโยคหรือวรรค)
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