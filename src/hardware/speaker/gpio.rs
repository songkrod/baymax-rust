use async_trait::async_trait;
use log::{info, debug};
use tokio::sync::Mutex;
use std::sync::Arc;
use std::time::Instant;

use super::interface::SpeakerBackend;

pub struct GpioSpeakerBackend {
    is_active: Arc<Mutex<bool>>,
}

impl GpioSpeakerBackend {
    pub fn new() -> Self {
        Self {
            is_active: Arc::new(Mutex::new(false)),
        }
    }
}

#[async_trait]
impl SpeakerBackend for GpioSpeakerBackend {
    async fn play(&self, data: &[u8]) -> Result<(), String> {
        let mut flag = self.is_active.lock().await;
        *flag = true;

        let t0 = Instant::now();

        info!("🔊 [GPIO] Playing {} bytes (real hardware expected)...", data.len());

        // TODO: Implement actual playback to GPIO-based audio device here
        // For now, just log and assume success
        // Example GPIO trigger could be implemented if needed

        let elapsed = t0.elapsed().as_secs_f32();
        info!("🔈 [GPIO] เสร็จสิ้นการเล่นเสียงใน {:.2} วินาที", elapsed);

        *flag = false;
        Ok(())
    }

    async fn is_busy(&self) -> bool {
        let busy = *self.is_active.lock().await;
        debug!("📡 [GpioSpeakerBackend] is_busy เรียกใช้ → {}", busy);
        busy
    }

    async fn play_beep_start(&self) {
        info!("🔔 [GPIO Beep] เริ่มฟัง (start)");
        // TODO: Replace with actual beep signal for start listening
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    async fn play_beep_end(&self) {
        info!("🔔 [GPIO Beep] จบการฟัง (end)");
        // TODO: Replace with actual beep signal for end listening
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }
}