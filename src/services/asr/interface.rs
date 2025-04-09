// src/services/asr/interface.rs

use async_trait::async_trait;
use std::error::Error;

/// Trait สำหรับระบบฟังเสียง (ASR - Automatic Speech Recognition)
#[async_trait]
pub trait ASRService: Send + Sync {
    /// รับ path ของไฟล์เสียง แล้วคืนข้อความที่ได้
    async fn transcribe(&self, audio_path: &str) -> Result<String, Box<dyn Error>>;
}