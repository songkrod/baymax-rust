use super::interface::TTSService;
use async_trait::async_trait;
use std::process::Command;
use std::error::Error;

pub struct LocalTTS;

#[async_trait]
impl TTSService for LocalTTS {
    async fn speak(&self, text: &str) -> Result<(), Box<dyn Error>> {
        let output = Command::new("espeak")
            .arg("-v")
            .arg("th")
            .arg(text)
            .output()?;

        if !output.status.success() {
            Err(format!("espeak failed: {:?}", output.stderr))?;
        }

        Ok(())
    }
}