use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use log::{info, warn, error, debug};
use async_trait::async_trait;

use super::interface::TTSService;
use super::google::GoogleTTS;
use super::espeak::ESpeakTTS;

pub struct SmartTTS {
    is_speaking: Arc<AtomicBool>,
    pub primary: Arc<dyn TTSService + Send + Sync>,
    pub fallback: Arc<dyn TTSService + Send + Sync>,
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

    pub async fn speak(&self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.is_speaking.store(true, Ordering::Relaxed);
        info!("🔈 [SmartTTS] กำลังพูด: '{}'", text);

        let result = match self.primary.speak(text).await {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("⚠️ Primary TTS failed: {}. Falling back...", e);
                match self.fallback.speak(text).await {
                    Ok(_) => Ok(()),
                    Err(fallback_err) => {
                        error!("❌ ทั้ง Primary และ Fallback TTS ล้มเหลว: '{}' => {}", text, fallback_err);
                        Err(fallback_err)
                    }
                }
            }
        };

        self.is_speaking.store(false, Ordering::Relaxed);
        result
    }

    pub async fn synthesize(&self, text: &str) -> Result<Vec<u8>, String> {
        info!("🎼 [SmartTTS] synthesize เริ่ม: '{}'", text);
        match self.primary.synthesize(text).await {
            Ok(data) => {
                debug!("✅ [SmartTTS] primary synthesize success: {} bytes", data.len());
                Ok(data)
            }
            Err(e) => {
                warn!("⚠️ Primary synthesize failed: {}. Falling back...", e);
                match self.fallback.synthesize(text).await {
                    Ok(data) => {
                        debug!("✅ fallback synthesize success: {} bytes (text: '{}')", data.len(), text);
                        Ok(data)
                    }
                    Err(fallback_err) => {
                        error!("❌ ทั้ง Primary และ Fallback synthesize ล้มเหลว: '{}' => {}", text, fallback_err);
                        Err(fallback_err)
                    }
                }
            }
        }
    }

    // Deprecated and unused in current architecture using TTSQueue
    #[allow(dead_code)]
    pub async fn speak_streamed(&self, _text: &str) -> Result<(), Box<dyn std::error::Error>> {
        warn!("⚠️ speak_streamed() is deprecated. Use enqueue_with_start_time() via TTSQueue instead.");
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

    async fn synthesize(&self, text: &str) -> Result<Vec<u8>, String> {
        self.synthesize(text).await
    }
}
