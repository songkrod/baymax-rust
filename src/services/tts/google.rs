use super::interface::TTSService;
use async_trait::async_trait;
use gcp_auth::AuthenticationManager;
use reqwest::{Client, header::AUTHORIZATION};
use base64::{engine::general_purpose, Engine};
use rodio::{Decoder, OutputStream, Sink};
use std::io::Cursor;
use std::error::Error;

pub struct GoogleTTS;

#[async_trait]
impl TTSService for GoogleTTS {
    async fn speak(&self, text: &str) -> Result<(), Box<dyn Error>> {
        let key_path = std::env::var("google_application_credentials")?;
        log::info!("🗝️ Using Google credentials at: {}", key_path);
        // let auth_manager = AuthenticationManager::new().await?;

        let auth_manager = AuthenticationManager::new().await.map_err(|e| {
            log::error!("❌ GCP Auth Init Failed: {:?}", e);
            e
        })?;

        let scopes = &["https://www.googleapis.com/auth/cloud-platform"];
        let token = auth_manager.get_token(scopes).await?;
        let bearer = format!("Bearer {}", token.as_str());

        let url = "https://texttospeech.googleapis.com/v1/text:synthesize";

        let request_body = serde_json::json!({
            "input": { "text": text },
            "voice": {
                "languageCode": "th-TH",
                "name": "th-TH-Chirp3-HD-Charon"
            },
            "audioConfig": {
                "audioEncoding": "MP3"
            }
        });

        let client = Client::new();
        let res = client.post(url)
            .header(AUTHORIZATION, bearer)
            .json(&request_body)
            .send()
            .await?;

        let json = res.json::<serde_json::Value>().await?;
        let audio_base64 = json["audioContent"]
            .as_str()
            .ok_or("No audio content in response")?;

        let audio_data = general_purpose::STANDARD.decode(audio_base64)?;
        let cursor = Cursor::new(audio_data);

        let (_stream, handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&handle)?;
        let source = Decoder::new(cursor)?;
        sink.append(source);
        sink.sleep_until_end();

        Ok(())
    }
}