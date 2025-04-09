use super::interface::TTSService;
use async_trait::async_trait;
use std::process::Command;
use std::error::Error;

pub struct ESpeakTTS;

#[async_trait]
impl TTSService for ESpeakTTS {
    async fn speak(&self, text: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let output = Command::new("espeak")
            .arg("-v")
            .arg("th")
            .arg(text)
            .output()?;

        if !output.status.success() {
            return Err(format!("espeak failed: {:?}", output.stderr).into());
        }

        Ok(())
    }
}