// 📁 src/services/tts/queue.rs

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use tokio::task;
use log::info;
use crate::services::tts::interface::TTSService;

pub struct TTSQueue {
    queue: Arc<Mutex<VecDeque<String>>>,
    is_speaking: Arc<AtomicBool>,
    preload_semaphore: Arc<tokio::sync::Semaphore>,
}

impl TTSQueue {
    pub fn new<T: TTSService + 'static + Send + Sync>(tts: Arc<T>, max_concurrent_preload: usize) -> Self {
        let queue = Arc::new(Mutex::new(VecDeque::<String>::new()));
        let is_speaking = Arc::new(AtomicBool::new(false));
        let preload_semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrent_preload));

        let queue_clone = Arc::clone(&queue);
        let is_speaking_clone = Arc::clone(&is_speaking);
        let preload_semaphore_clone = Arc::clone(&preload_semaphore);

        task::spawn(async move {
            loop {
                let next_text = {
                    let mut q = queue_clone.lock().unwrap();
                    q.pop_front()
                };

                if let Some(text) = next_text {
                    // รอถ้ามี preload เกิน limit
                    let permit = preload_semaphore_clone.clone().acquire_owned().await.unwrap();
                    is_speaking_clone.store(true, Ordering::Relaxed);

                    info!("🔊 [TTSQueue] กำลังพูด: {}", text);

                    let _ = tts.speak(&text).await;

                    // ปล่อย slot ให้ preload ตัวถัดไป
                    drop(permit);
                    is_speaking_clone.store(false, Ordering::Relaxed);
                } else {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        });

        Self {
            queue,
            is_speaking,
            preload_semaphore,
        }
    }

    pub fn enqueue(&self, text: &str) {
        let mut q = self.queue.lock().unwrap();
        info!("📥 เพิ่มเข้า TTS Queue: {} (len={})", text, q.len());
        q.push_back(text.to_string());
    }

    pub fn is_speaking(&self) -> bool {
        self.is_speaking.load(Ordering::Relaxed)
    }
}
