use super::interface::ASRService;
use async_trait::async_trait;
use reqwest::Client;
use std::error::Error;
use std::fs;
use log::info;

pub struct WhisperCloud;

#[async_trait]
impl ASRService for WhisperCloud {
    async fn transcribe(&self, path: &str) -> Result<String, Box<dyn Error>> {
        let api_key = std::env::var("openai_api_key")?;
        let client = Client::new();

        // อ่านไฟล์เสียงจาก path ที่กำหนด
        let audio_data = fs::read(path)?;
        info!("🎧 Reading audio file from: {}", path);

        let part = reqwest::multipart::Form::new()
            .part("file", reqwest::multipart::Part::bytes(audio_data).file_name("baymax_audio.wav"))
            .text("model", "whisper-1");

        let res = client
            .post("https://api.openai.com/v1/audio/transcriptions")
            .bearer_auth(api_key)
            .multipart(part)
            .send()
            .await?;

        let json: serde_json::Value = res.json().await?;
        let text = json["text"].as_str().unwrap_or("").to_string();
        Ok(text)
    }
}