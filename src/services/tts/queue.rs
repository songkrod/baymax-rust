use std::collections::VecDeque;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use tokio::task;
use log::{info};
use crate::services::tts::interface::TTSService;

pub struct TTSQueue {
    queue: Arc<Mutex<VecDeque<String>>>,
    is_speaking: Arc<AtomicBool>,
}

impl TTSQueue {
    pub fn new<T: TTSService + 'static + Send + Sync>(tts: Arc<T>) -> Self {
        let queue = Arc::new(Mutex::new(VecDeque::<String>::new()));
        let is_speaking = Arc::new(AtomicBool::new(false));
        let queue_clone = Arc::clone(&queue);
        let is_speaking_clone = Arc::clone(&is_speaking);

        task::spawn(async move {
            loop {
                if let Some(next_text) = {
                    let mut q = queue_clone.lock().unwrap();
                    q.pop_front()
                } {
                    is_speaking_clone.store(true, Ordering::Relaxed);
                    info!("🔊 [TTSQueue] กำลังพูด: {}", next_text);

                    let _ = tts.speak(&next_text).await;
                    is_speaking_clone.store(false, Ordering::Relaxed);
                } else {
                    // sleep นิดหน่อยเพื่อลด CPU usage
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        });

        Self { queue, is_speaking }
    }

    pub fn enqueue(&self, text: &str) {
        info!("📥 เพิ่มเข้า TTS Queue: {}", text);
        let mut q = self.queue.lock().unwrap();
        q.push_back(text.to_string());
    }

    pub fn is_speaking(&self) -> bool {
        self.is_speaking.load(Ordering::Relaxed)
    }
}