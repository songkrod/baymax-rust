// 📁 src/services/llm/open_ai.rs

use super::interface::{LLMService, LLMStreamable};
use async_trait::async_trait;
use reqwest::{Client, header};
use serde_json::{json, Value};
use tokio_stream::StreamExt;

pub struct OpenAIGPT;

#[async_trait]
impl LLMService for OpenAIGPT {
    async fn chat(&self, input: &str) -> Result<String, Box<dyn std::error::Error>> {
        let api_key = std::env::var("openai_api_key")?;
        let client = Client::new();

        let body = json!({
            "model": "gpt-3.5-turbo",
            "messages": [{"role": "user", "content": input}],
        });

        let res = client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .await?;

        let json: Value = res.json().await?;
        let reply = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(reply)
    }
}

#[async_trait]
impl LLMStreamable for OpenAIGPT {
    async fn stream_chat<F>(&self, input: &str, mut on_chunk: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut(String) + Send + 'static,
    {
        let api_key = std::env::var("openai_api_key")?;
        let client = Client::new();

        let body = json!({
            "model": "gpt-3.5-turbo",
            "stream": true,
            "temperature": 0.8,
            "messages": [
                {"role": "user", "content": input}
            ]
        });

        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(api_key)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&body)
            .send()
            .await?;

        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let chunk_str = String::from_utf8_lossy(&chunk);

            for line in chunk_str.lines() {
                if line.starts_with("data: ") {
                    let json_part = &line[6..];
                    if json_part.trim() == "[DONE]" {
                        break;
                    }

                    if let Ok(json) = serde_json::from_str::<Value>(json_part) {
                        if let Some(text) = json["choices"][0]["delta"]["content"].as_str() {
                            on_chunk(text.to_string());
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
