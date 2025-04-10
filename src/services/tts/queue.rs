// 📁 src/services/tts/queue.rs

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use tokio::task;
use log::info;
use crate::services::tts::interface::TTSService;

pub struct TTSQueue {
    queue: Arc<Mutex<VecDeque<String>>>,
    is_speaking: Arc<AtomicBool>,
    max_preload: usize,
}

impl TTSQueue {
    pub fn new<T: TTSService + 'static + Send + Sync>(tts: Arc<T>, max_preload: usize) -> Self {
        let queue = Arc::new(Mutex::new(VecDeque::<String>::new()));
        let is_speaking = Arc::new(AtomicBool::new(false));
        let queue_clone = Arc::clone(&queue);
        let is_speaking_clone = Arc::clone(&is_speaking);

        task::spawn(async move {
            loop {
                let next_text = {
                    let mut q = queue_clone.lock().unwrap();
                    q.pop_front()
                };

                if let Some(text) = next_text {
                    is_speaking_clone.store(true, Ordering::Relaxed);
                    info!("🔊 [TTSQueue] กำลังพูด: {}", text);

                    let _ = tts.speak(&text).await;
                    is_speaking_clone.store(false, Ordering::Relaxed);
                } else {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        });

        Self { queue, is_speaking, max_preload }
    }

    pub fn enqueue(&self, text: &str) {
        let mut q = self.queue.lock().unwrap();
        if q.len() < self.max_preload {
            info!("📥 เพิ่มเข้า TTS Queue: {}", text);
            q.push_back(text.to_string());
        } else {
            info!("🚫 ตัด chunk เพราะ queue เต็ม ({}): {}", self.max_preload, text);
        }
    }

    pub fn is_speaking(&self) -> bool {
        self.is_speaking.load(Ordering::Relaxed)
    }
}
