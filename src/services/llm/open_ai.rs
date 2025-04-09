use super::interface::LLMService;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

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

        let json: serde_json::Value = res.json().await?;
        let reply = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(reply)
    }
}