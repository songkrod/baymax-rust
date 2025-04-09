use std::sync::Arc;
use log::{info, warn, error};

use super::interface::ASRService;
use super::whisper_local::WhisperLocal;
use super::whisper_cloud::WhisperCloud;
use crate::utils::vad::record_and_trim;

/// ระบบแปลงเสียงเป็นข้อความแบบสลับ backend ได้ (pluggable)
pub struct SmartASR {
    primary: Arc<dyn ASRService + Send + Sync>,
    fallback: Arc<dyn ASRService + Send + Sync>,
}

impl SmartASR {
    /// สร้างระบบ ASR ที่เลือก backend ได้ผ่าน ENV
    pub fn new() -> Self {
        let backend = std::env::var("asr_backend").unwrap_or_else(|_| "local".to_string());

        let primary: Arc<dyn ASRService + Send + Sync> = match backend.as_str() {
            "cloud" => {
                info!("🧠 [ASR] ใช้งาน Whisper Cloud (OpenAI API) เป็นระบบหลัก");
                Arc::new(WhisperCloud)
            }
            "local" => {
                info!("🧠 [ASR] ใช้งาน Whisper Local (Python) เป็นระบบหลัก");
                Arc::new(WhisperLocal)
            }
            unknown => {
                warn!("❗ [ASR] ค่า backend '{}' ไม่รู้จัก → fallback เป็น local", unknown);
                Arc::new(WhisperLocal)
            }
        };

        let fallback: Arc<dyn ASRService + Send + Sync> = Arc::new(WhisperLocal);

        Self { primary, fallback }
    }

    /// ฟังเสียงแล้วแปลงเป็นข้อความ พร้อม fallback หาก primary ล้มเหลว
    pub async fn transcribe(&self, path: &str) -> Result<String, Box<dyn std::error::Error>> {
        info!("🧠 [ASR] เริ่มถอดเสียงจากไฟล์: {}", path);
        match self.primary.transcribe(path).await {
            Ok(text) => {
                info!("✅ [Primary] สำเร็จ: \"{}\"", text);
                Ok(text)
            }
            Err(e) => {
                warn!("⚠️ [Primary] ล้มเหลว: {} → ใช้ fallback", e);
                match self.fallback.transcribe(path).await {
                    Ok(text) => {
                        info!("✅ [Fallback] สำเร็จ: \"{}\"", text);
                        Ok(text)
                    }
                    Err(e) => {
                        error!("❌ [Fallback] ถอดเสียงล้มเหลว: {}", e);
                        Err(e)
                    }
                }
            }
        }
    }

    /// ฟังเสียง (ผ่านไมค์ + VAD) แล้วแปลงเป็นข้อความ
    pub async fn listen(
        &self,
        input_path: &str,
        max_duration_ms: Option<u64>,
        silence_timeout_ms: Option<u64>,
    ) -> Option<String> {
        let max = max_duration_ms.unwrap_or(10_000);
        let silence = silence_timeout_ms.unwrap_or(1000);

        info!("🎙️ [ASR] เริ่มบันทึกเสียง VAD (max: {}ms, silence: {}ms)", max, silence);

        if let Err(e) = record_and_trim(input_path, max, silence) {
            error!("❌ [ASR] ไม่สามารถบันทึกเสียงได้: {}", e);
            return None;
        }

        match self.transcribe(input_path).await {
            Ok(text) if !text.trim().is_empty() => Some(text),
            Ok(_) => {
                warn!("📭 [ASR] ไม่มีคำพูดที่ตรวจจับได้");
                None
            }
            Err(e) => {
                error!("🧠 [ASR] ถอดเสียงล้มเหลว: {}", e);
                None
            }
        }
    }
}
