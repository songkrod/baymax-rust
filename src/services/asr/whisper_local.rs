use super::interface::ASRService;
use async_trait::async_trait;
use std::error::Error;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use log::{info, error};

pub struct WhisperLocal;

#[async_trait]
impl ASRService for WhisperLocal {
    async fn transcribe(&self, path: &str) -> Result<String, Box<dyn Error>> {
        if !Path::new(path).exists() {
            return Err("❌ WhisperLocal: Provided audio file does not exist.".into());
        }

        info!("🎧 [WhisperLocal] Transcribing from file: {}", path);

        let mut child = Command::new("python3")
            .arg("scripts/transcribe.py")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        {
            let stdin = child.stdin.as_mut().ok_or("Failed to open stdin")?;
            stdin.write_all(format!("{}\n", path).as_bytes())?;
        }

        let stdout = child.stdout.take().ok_or("Failed to open stdout")?;
        let reader = BufReader::new(stdout);
        let mut transcript = None;

        for line in reader.lines() {
            let line = line?;
            info!("🐍 Python says: {}", line);

            if line.starts_with("ERROR") {
                error!("❌ Whisper Local Error: {}", line);
                return Err(line.into());
            }

            if line.starts_with("TRANSCRIPT: ") {
                transcript = Some(line.trim_start_matches("TRANSCRIPT: ").to_string());
                break; // <<< นี่แหละของจริง
            }
        }

        match transcript {
            Some(text) => Ok(text),
            None => Err("WhisperLocal: No transcript received.".into()),
        }
    }
}