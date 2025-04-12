use async_trait::async_trait;
use log::{info, error, debug};
use tokio::process::Command;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::time::Instant;
use super::interface::SpeakerBackend;

pub struct OsSpeakerBackend {
    is_active: Arc<Mutex<bool>>,
}

impl OsSpeakerBackend {
    pub fn new() -> Self {
        Self {
            is_active: Arc::new(Mutex::new(false)),
        }
    }
}

#[async_trait]
impl SpeakerBackend for OsSpeakerBackend {
    async fn play(&self, data: &[u8]) -> Result<(), String> {
        use tempfile::NamedTempFile;
        use std::io::Write;

        debug!("🔉 ขนาด mp3 ที่รับเข้ามา: {} bytes", data.len());

        let mut temp = NamedTempFile::new().map_err(|e| e.to_string())?;
        temp.write_all(data).map_err(|e| e.to_string())?;
        let path = temp.path().to_str().unwrap().to_string();

        let player = if cfg!(target_os = "macos") {
            "afplay"
        } else {
            "mpg123"
        };

        info!("▶️ เรียก player '{}': {}", player, path);

        {
            let mut flag = self.is_active.lock().await;
            *flag = true;
        }

        let t0 = Instant::now();

        let status = Command::new(player)
            .arg(&path)
            .output()
            .await
            .map_err(|e| format!("ไม่สามารถเรียก player ได้: {}", e))?;

        let elapsed = t0.elapsed().as_secs_f32();
        info!("🔈 เสร็จสิ้นการเล่นเสียงใน {:.2} วินาที", elapsed);

        {
            let mut flag = self.is_active.lock().await;
            *flag = false;
        }

        if status.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&status.stderr);
            error!("❌ player exited with code {:?}, stderr: {}", status.status.code(), stderr);
            Err(format!("player exited with {:?}", status.status.code()))
        }
    }

    async fn is_busy(&self) -> bool {
        let busy = *self.is_active.lock().await;
        info!("📡 [OsSpeakerBackend] is_busy เรียกใช้ → {}", busy);
        busy
    }
}