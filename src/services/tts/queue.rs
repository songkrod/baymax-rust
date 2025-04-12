use std::collections::VecDeque;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::time::{Instant, Duration};
use tokio::task;
use tokio::time::sleep;
use log::{info, debug, error};

use crate::services::tts::interface::TTSService;
use crate::hardware::speaker::interface::SpeakerBackend;

pub struct TTSQueue {
    queue: Arc<Mutex<VecDeque<TTSJob>>>,
    speaker: Arc<dyn SpeakerBackend>,
    tts: Arc<dyn TTSService>,
    preload_semaphore: Arc<tokio::sync::Semaphore>,
    is_speaking: Arc<AtomicBool>,
}

struct TTSJob {
    text: String,
    mp3_data: Option<Vec<u8>>,
    start_time: Option<Instant>,
    with_pause_after: bool,
}

impl TTSQueue {
    pub fn new(
        tts: Arc<dyn TTSService>,
        speaker: Arc<dyn SpeakerBackend>,
        max_concurrent_preload: usize,
    ) -> Self {
        let queue = Arc::new(Mutex::new(VecDeque::<TTSJob>::new()));
        let preload_semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrent_preload));
        let is_speaking = Arc::new(AtomicBool::new(false));

        let queue_clone = Arc::clone(&queue);
        let preload_semaphore_clone = Arc::clone(&preload_semaphore);
        let speaker_clone = Arc::clone(&speaker);
        let is_speaking_clone = Arc::clone(&is_speaking);

        task::spawn(async move {
            loop {
                let next_job = {
                    let mut q = queue_clone.lock().unwrap();
                    q.pop_front()
                };

                if let Some(job) = next_job {
                    debug!("🧾 ประมวลผล job: '{}' (มี mp3 = {})", job.text, job.mp3_data.is_some());
                    if let Some(mp3) = job.mp3_data {
                        is_speaking_clone.store(true, Ordering::Relaxed);

                        let permit = preload_semaphore_clone.clone().acquire_owned().await.unwrap();

                        if let Some(t0) = job.start_time {
                            let elapsed = t0.elapsed().as_secs_f32();
                            info!("📈 [Perf] เริ่มพูดหลัง GPT ใช้เวลา: {:.2}s", elapsed);
                        }

                        info!("🔊 [TTSQueue] กำลังพูด: {}", job.text);
                        if let Err(e) = speaker_clone.play(&mp3).await {
                            error!("❌ เล่นเสียงล้มเหลว: {}", e);
                        }

                        let dur = estimate_mp3_duration(&mp3);
                        debug!("🕒 ประมาณเวลาพูด: {:.2}s", dur.as_secs_f32());
                        sleep(dur).await;

                        drop(permit);
                    } else {
                        error!("⚠️ ไม่มี mp3_data สำหรับ '{}' ข้าม...", job.text);
                    }

                    if queue_clone.lock().unwrap().is_empty() {
                        is_speaking_clone.store(false, Ordering::Relaxed);
                        debug!("📭 Queue ว่างแล้ว ปิด flag is_speaking");
                    }
                } else {
                    is_speaking_clone.store(false, Ordering::Relaxed);
                    sleep(Duration::from_millis(100)).await;
                }
            }
        });

        Self {
            queue,
            speaker,
            tts,
            preload_semaphore,
            is_speaking,
        }
    }

    pub fn enqueue(&self, text: &str) {
        self.enqueue_with_start_time_and_pause(text, None, false);
    }

    pub fn enqueue_with_start_time(&self, text: &str, start_time: Option<Instant>) {
        self.enqueue_with_start_time_and_pause(text, start_time, false);
    }

    pub fn enqueue_with_start_time_and_pause(&self, text: &str, start_time: Option<Instant>, with_pause_after: bool) {
        let text = text.to_string();
        let queue = Arc::clone(&self.queue);
        let preload_semaphore = Arc::clone(&self.preload_semaphore);
        let tts = Arc::clone(&self.tts);

        info!("📥 เพิ่มเข้า TTS Queue (รอ preload): {} (pause = {})", text, with_pause_after);

        task::spawn(async move {
            debug!("🌀 Preloading เสียงสำหรับ: {}", text);
            let mp3_data = match tts.synthesize(&text).await {
                Ok(data) => {
                    debug!("✅ สร้างเสียงสำเร็จ: {} ({} bytes)", text, data.len());
                    Some(data)
                }
                Err(err) => {
                    error!("❌ สร้างเสียงล้มเหลว: {} => {}", text, err);
                    None
                }
            };

            let job = TTSJob { text: text.clone(), mp3_data, start_time, with_pause_after };
            let mut q = queue.lock().unwrap();
            debug!("📦 เพิ่มเข้า VecDeque: {} (ตอนนี้มี {} job)", text, q.len() + 1);
            q.push_back(job);
        });
    }

    pub async fn enqueue_and_wait(&self, text: &str) {
        self.enqueue(text);

        let mut waited = 0;
        while !self.is_speaking.load(Ordering::Relaxed) && waited < 2000 {
            sleep(Duration::from_millis(50)).await;
            waited += 50;
        }

        info!("✅ speaker เริ่มพูดแล้ว หลังรอ {}ms", waited);

        while self.is_speaking.load(Ordering::Relaxed) {
            sleep(Duration::from_millis(200)).await;
        }

        info!("🔇 เสร็จสิ้นการพูดทั้งหมด");
    }

    pub async fn is_speaking(&self) -> bool {
        self.is_speaking.load(Ordering::Relaxed)
    }

    pub fn block_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.is_speaking)
    }
}

fn estimate_mp3_duration(data: &[u8]) -> Duration {
    let kbps = 64.0; // สมมุติ SmartTTS encode ที่ 64 kbps
    let bytes_per_sec = (kbps * 1000.0 / 8.0); // -> 8000 bytes/sec
    let secs = data.len() as f64 / bytes_per_sec;
    Duration::from_secs_f64(secs)
}
