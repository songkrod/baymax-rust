use crate::services::tts::interface::TTSService;
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine};
use chrono::{Utc, Duration};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::Client;
use rodio::{Decoder, OutputStream, Sink};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Cursor;
use std::path::Path;

#[derive(Deserialize)]
struct ServiceAccountKey {
    client_email: String,
    private_key: String,
    token_uri: String,
}

#[derive(Serialize)]
struct Claims {
    iss: String,
    scope: String,
    aud: String,
    exp: usize,
    iat: usize,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

pub struct GoogleTTS;

#[async_trait]
impl TTSService for GoogleTTS {
    async fn speak(&self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let audio_data = self.synthesize(text).await?;
        let cursor = Cursor::new(audio_data);
        let (_stream, handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&handle)?;
        let source = Decoder::new(cursor)?;
        sink.append(source);
        sink.sleep_until_end();
        Ok(())
    }

    async fn synthesize(&self, text: &str) -> Result<Vec<u8>, String> {
        let key_path = std::env::var("google_application_credentials")
            .map_err(|e| format!("env error: {}", e))?;
        let key_str = fs::read_to_string(Path::new(&key_path)).map_err(|e| format!("read error: {}", e))?;
        let key: ServiceAccountKey = serde_json::from_str(&key_str).map_err(|e| format!("json error: {}", e))?;

        let now = Utc::now();
        let claims = Claims {
            iss: key.client_email.clone(),
            scope: "https://www.googleapis.com/auth/cloud-platform".to_string(),
            aud: key.token_uri.clone(),
            iat: now.timestamp() as usize,
            exp: (now + Duration::minutes(60)).timestamp() as usize,
        };

        let encoding_key = EncodingKey::from_rsa_pem(key.private_key.as_bytes())
            .map_err(|e| format!("rsa error: {}", e))?;
        let jwt = encode(&Header::new(Algorithm::RS256), &claims, &encoding_key)
            .map_err(|e| format!("jwt error: {}", e))?;

        let client = Client::new();
        let res = client.post(&key.token_uri)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", &jwt),
            ])
            .send()
            .await.map_err(|e| format!("auth error: {}", e))?;

        let token_data: TokenResponse = res.json().await.map_err(|e| format!("parse token error: {}", e))?;
        let bearer = format!("Bearer {}", token_data.access_token);

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

        let res = client.post(url)
            .header("Authorization", bearer)
            .json(&request_body)
            .send()
            .await.map_err(|e| format!("tts request error: {}", e))?;

        let json = res.json::<serde_json::Value>().await.map_err(|e| format!("tts response error: {}", e))?;
        let audio_base64 = json["audioContent"].as_str().ok_or("missing audioContent")?;
        general_purpose::STANDARD.decode(audio_base64).map_err(|e| format!("base64 decode error: {}", e))
    }
}
