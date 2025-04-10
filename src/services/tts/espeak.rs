// 📁 src/services/tts/espeak.rs

use async_trait::async_trait;
use std::process::{Command, Stdio};
use std::io::{Read, Write};
use std::fs::File;
use tempfile::NamedTempFile;
use log::{info, error};
use super::interface::TTSService;

pub struct ESpeakTTS;

#[async_trait]
impl TTSService for ESpeakTTS {
    async fn speak(&self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("🗣️ (eSpeak) {}", text);
        Command::new("espeak")
            .arg(text)
            .spawn()?
            .wait()?;
        Ok(())
    }

    async fn synthesize(&self, text: &str) -> Result<Vec<u8>, String> {
        // 1. สร้าง wav temp file
        let wav = NamedTempFile::new().map_err(|e| e.to_string())?;
        let wav_path = wav.path().to_string_lossy().to_string();

        // 2. เรียก espeak ให้สร้างไฟล์ wav
        let espeak_status = Command::new("espeak")
            .arg(text)
            .arg("-w")
            .arg(&wav_path)
            .stdout(Stdio::null())
            .status()
            .map_err(|e| e.to_string())?;

        if !espeak_status.success() {
            return Err("espeak failed".to_string());
        }

        // 3. แปลง wav → mp3 ด้วย lame
        let mp3 = NamedTempFile::new().map_err(|e| e.to_string())?;
        let mp3_path = mp3.path().to_string_lossy().to_string();

        let lame_status = Command::new("lame")
            .arg(&wav_path)
            .arg(&mp3_path)
            .stdout(Stdio::null())
            .status()
            .map_err(|e| e.to_string())?;

        if !lame_status.success() {
            return Err("lame failed".to_string());
        }

        // 4. โหลดไฟล์ mp3 กลับมาเป็น Vec<u8>
        let mut file = File::open(mp3.path()).map_err(|e| e.to_string())?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).map_err(|e| e.to_string())?;

        Ok(buffer)
    }
}