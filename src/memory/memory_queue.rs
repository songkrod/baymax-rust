// src/memory/memory_queue.rs
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use crate::memory::memory_manager::MemoryManager;

#[derive(Clone)]
pub struct MemoryQueue {
    sender: mpsc::Sender<(String, String)>,
}

impl MemoryQueue {
    pub fn new(memory_manager: Arc<Mutex<MemoryManager>>) -> Self {
        let (tx, mut rx) = mpsc::channel::<(String, String)>(32);

        tokio::spawn(async move {
            while let Some((key, value)) = rx.recv().await {
                let mut manager = memory_manager.lock().await;
                manager.add_memory(&key, &value).await;
            }
        });

        Self { sender: tx }
    }

    pub async fn enqueue(&self, key: &str, value: &str) {
        if let Err(e) = self.sender.send((key.to_string(), value.to_string())).await {
            eprintln!("❌ Failed to enqueue memory: {}", e);
        }
    }
}