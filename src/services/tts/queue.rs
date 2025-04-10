use std::collections::VecDeque;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use tokio::task;
use tokio::time::{sleep, Duration};
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
                    if let Some(mp3) = job.mp3_data {
                        is_speaking_clone.store(true, Ordering::Relaxed);

                        let permit = preload_semaphore_clone.clone().acquire_owned().await.unwrap();

                        info!("🔊 [TTSQueue] กำลังพูด: {}", job.text);
                        if let Err(e) = speaker_clone.play(&mp3).await {
                            error!("❌ เล่นเสียงล้มเหลว: {}", e);
                        }

                        sleep(Duration::from_millis(300)).await;
                        drop(permit);
                    } else {
                        error!("⚠️ ไม่มี mp3_data สำหรับ '{}', ข้าม...", job.text);
                    }

                    // จบ chunk นี้ให้ clear flag
                    if queue_clone.lock().unwrap().is_empty() {
                        is_speaking_clone.store(false, Ordering::Relaxed);
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
        let text = text.to_string();
        let queue = Arc::clone(&self.queue);
        let preload_semaphore = Arc::clone(&self.preload_semaphore);
        let tts = Arc::clone(&self.tts);

        info!("📥 เพิ่มเข้า TTS Queue: {}", text);

        task::spawn(async move {
            debug!("🌀 Preloading เสียงสำหรับ: {}", text);
            let mp3_data = tts.synthesize(&text).await.ok();
            let job = TTSJob { text, mp3_data };

            let mut q = queue.lock().unwrap();
            q.push_back(job);
        });
    }

    pub async fn enqueue_and_wait(&self, text: &str) {
        self.enqueue(text);

        // Wait until speaking flag is true (speaker may lag slightly)
        let mut waited = 0;
        while !self.is_speaking.load(Ordering::Relaxed) && waited < 2000 {
            sleep(Duration::from_millis(50)).await;
            waited += 50;
        }

        info!("✅ speaker เริ่มพูดแล้ว หลังรอ {}ms", waited);
    }

    pub async fn is_speaking(&self) -> bool {
        self.is_speaking.load(Ordering::Relaxed)
    }

    pub fn block_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.is_speaking)
    }
}