use std::collections::VecDeque;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, AtomicUsize, Ordering}};
use std::time::{Instant, Duration};
use tokio::task;
use tokio::sync::{Notify, watch};
use log::{info, debug, error, warn};

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
    start_notify: Arc<Notify>,
    wake_notify: Arc<Notify>,
    done_notify: Arc<Notify>,
    done_signal_tx: watch::Sender<()>,
}

impl TTSQueue {
    pub fn new(
        tts: Arc<dyn TTSService>,
        speaker: Arc<dyn SpeakerBackend>,
        max_concurrent_preload: usize,
    ) -> (Self, watch::Receiver<()>) {
        let queue = Arc::new(Mutex::new(VecDeque::<TTSJob>::new()));
        let preload_semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrent_preload));
        let is_speaking = Arc::new(AtomicBool::new(false));
        let start_notify = Arc::new(Notify::new());
        let wake_notify = Arc::new(Notify::new());
        let done_notify = Arc::new(Notify::new());
        let (done_signal_tx, done_signal_rx) = watch::channel(());

        let queue_clone = Arc::clone(&queue);
        let speaker_clone = Arc::clone(&speaker);
        let is_speaking_clone = Arc::clone(&is_speaking);
        let start_notify_clone = Arc::clone(&start_notify);
        let wake_notify_clone = Arc::clone(&wake_notify);
        let done_notify_clone = Arc::clone(&done_notify);
        let done_signal_tx_clone = done_signal_tx.clone();

        task::spawn(async move {
            loop {
                let job = loop {
                    let maybe_job = {
                        let mut q = queue_clone.lock().unwrap();
                        q.pop_front()
                    };
                    if let Some(job) = maybe_job {
                        break job;
                    } else {
                        wake_notify_clone.notified().await;
                    }
                };

                debug!("🧾 ประมวลผล job[{}]: '{}' (mp3 = {})", job.index, job.text, job.mp3_data.is_some());

                if let Some(mp3) = job.mp3_data.clone() {
                    is_speaking_clone.store(true, Ordering::Relaxed);
                    start_notify_clone.notify_waiters();

                    if let Some(t0) = job.start_time {
                        let elapsed = t0.elapsed().as_secs_f32();
                        info!("📈 [Perf] เริ่มพูดหลัง GPT ใช้เวลา: {:.2}s", elapsed);
                    }

                    info!("🔊 [TTSQueue] พูด: {}", job.text);
                    let result = speaker_clone.play(&mp3).await;
                    if let Err(e) = result {
                        error!("❌ เล่นเสียงล้มเหลว: {}", e);
                    }

                    is_speaking_clone.store(false, Ordering::Relaxed);
                    debug!("🔇 พูดจบ job[{}]: {}", job.index, job.text);

                    let _ = done_signal_tx_clone.send(());
                    done_notify_clone.notify_waiters();
                } else {
                    error!("⚠️ ไม่มี mp3_data สำหรับ '{}'", job.text);
                }
            }
        });

        (
            Self {
                queue,
                speaker,
                tts,
                preload_semaphore,
                is_speaking,
                start_notify,
                wake_notify,
                done_notify,
                done_signal_tx,
            },
            done_signal_rx,
        )
    }

    pub fn enqueue(&self, text: &str) {
        self.enqueue_with_start_time(text, None);
    }

    pub fn enqueue_with_start_time(&self, text: &str, start_time: Option<Instant>) {
        let index = JOB_COUNTER.fetch_add(1, Ordering::SeqCst);
        let text = text.to_string();
        let queue = Arc::clone(&self.queue);
        let tts = Arc::clone(&self.tts);
        let wake_notify = Arc::clone(&self.wake_notify);

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
            wake_notify.notify_one();
        });
    }

    pub async fn enqueue_and_wait(&self, text: &str) {
        self.enqueue(text);

        let timeout = Duration::from_millis(2000);
        let start = tokio::time::timeout(timeout, self.start_notify.notified()).await;

        match start {
            Ok(_) => info!("✅ speaker เริ่มพูดตามสัญญาณ notify แล้ว"),
            Err(_) => warn!("⌛ timeout: รอ speaker เริ่มพูดเกิน 2s แล้ว"),
        }

        self.wait_until_done().await;

        info!("🔇 จบการพูด");
    }

    pub async fn wait_until_done(&self) {
        while self.is_speaking().await {
            self.done_notify.notified().await;
        }
    }

    pub async fn is_speaking(&self) -> bool {
        self.is_speaking.load(Ordering::Relaxed)
    }

    pub fn block_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.is_speaking)
    }

    pub async fn play_beep_start(&self) {
        self.speaker.play_beep_start().await;
    }

    pub async fn play_beep_end(&self) {
        self.speaker.play_beep_end().await;
    }

    pub fn subscribe_done_event(&self) -> watch::Receiver<()> {
        self.done_signal_tx.subscribe()
    }
}

fn estimate_mp3_duration(data: &[u8]) -> Duration {
    let kbps = 64.0;
    let bytes_per_sec = kbps * 1000.0 / 8.0;
    Duration::from_secs_f64(data.len() as f64 / bytes_per_sec)
}