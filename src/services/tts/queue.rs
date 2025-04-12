use std::collections::VecDeque;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, AtomicUsize, Ordering}};
use std::time::{Instant, Duration};
use tokio::task;
use tokio::time::sleep;
use log::{info, debug, error};

use crate::services::tts::interface::TTSService;
use crate::hardware::speaker::interface::SpeakerBackend;

static JOB_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone)]
struct TTSJob {
    index: usize,
    text: String,
    mp3_data: Option<Arc<Vec<u8>>>,
    start_time: Option<Instant>,
    with_pause_after: bool,
}

pub struct TTSQueue {
    queue: Arc<Mutex<VecDeque<TTSJob>>>,
    speaker: Arc<dyn SpeakerBackend>,
    tts: Arc<dyn TTSService>,
    preload_semaphore: Arc<tokio::sync::Semaphore>,
    is_speaking: Arc<AtomicBool>,
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
        let speaker_clone = Arc::clone(&speaker);
        let is_speaking_clone = Arc::clone(&is_speaking);

        task::spawn(async move {
            loop {
                let job = {
                    let mut q = queue_clone.lock().unwrap();
                    q.pop_front()
                };

                if let Some(job) = job {
                    debug!("🧾 ประมวลผล job[{}]: '{}' (mp3 = {})", job.index, job.text, job.mp3_data.is_some());

                    if let Some(mp3) = job.mp3_data.clone() {
                        is_speaking_clone.store(true, Ordering::Relaxed);

                        if let Some(t0) = job.start_time {
                            let elapsed = t0.elapsed().as_secs_f32();
                            info!("📈 [Perf] เริ่มพูดหลัง GPT ใช้เวลา: {:.2}s", elapsed);
                        }

                        info!("🔊 [TTSQueue] พูด: {}", job.text);
                        if let Err(e) = speaker_clone.play(&mp3).await {
                            error!("❌ เล่นเสียงล้มเหลว: {}", e);
                        }

                        // if job.with_pause_after {
                        //     debug!("🕒 พัก 50ms");
                        //     sleep(Duration::from_millis(50)).await;
                        // }
                    } else {
                        error!("⚠️ ไม่มี mp3_data สำหรับ '{}'", job.text);
                    }

                    if queue_clone.lock().unwrap().is_empty() {
                        is_speaking_clone.store(false, Ordering::Relaxed);
                        debug!("📭 Queue ว่างแล้ว");
                    }
                } else {
                    is_speaking_clone.store(false, Ordering::Relaxed);
                    sleep(Duration::from_millis(10)).await;
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
        self.enqueue_with_start_time(text, None);
    }

    pub fn enqueue_with_start_time(&self, text: &str, start_time: Option<Instant>) {
        let index = JOB_COUNTER.fetch_add(1, Ordering::SeqCst);
        let text = text.to_string();
        let queue = Arc::clone(&self.queue);
        let tts = Arc::clone(&self.tts);

        info!("📥 เพิ่มเข้า TTS Queue: {}", text);

        task::spawn(async move {
            debug!("🌀 สร้างเสียง: {}", text);
            let mp3_data = match tts.synthesize(&text).await {
                Ok(data) => {
                    debug!("✅ เสร็จ: {} ({} bytes)", text, data.len());
                    Some(Arc::new(data))
                }
                Err(err) => {
                    error!("❌ สร้างเสียงล้มเหลว: {}", err);
                    None
                }
            };

            let job = TTSJob {
                index,
                text: text.clone(),
                mp3_data,
                start_time,
                with_pause_after: true,
            };

            let mut q = queue.lock().unwrap();
            debug!("📦 เข้า queue ({} jobs): {}", q.len() + 1, text);
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

        info!("✅ speaker เริ่มพูดหลังรอ {}ms", waited);

        while self.is_speaking.load(Ordering::Relaxed) {
            sleep(Duration::from_millis(200)).await;
        }

        info!("🔇 จบการพูด");
    }

    pub async fn is_speaking(&self) -> bool {
        self.is_speaking.load(Ordering::Relaxed)
    }

    pub fn block_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.is_speaking)
    }
}

fn estimate_mp3_duration(data: &[u8]) -> Duration {
    let kbps = 64.0;
    let bytes_per_sec = kbps * 1000.0 / 8.0;
    Duration::from_secs_f64(data.len() as f64 / bytes_per_sec)
}
