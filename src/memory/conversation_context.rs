// 📁 src/memory/conversation_context.rs
use std::fs::{read_to_string, write};
use std::sync::Mutex;
use std::env;
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Serialize, Deserialize, Clone)]
pub struct Dialogue {
    pub user: String,
    pub bot: String,
}

pub struct ConversationContext {
    path: String,
    memory: Mutex<Vec<Dialogue>>, // ใช้ Vec<Dialogue> แทน tuple
}

impl ConversationContext {
    pub fn new() -> Self {
        let path = env::var("context_cache_path").unwrap_or_else(|_| "src/data/memory/context_cache.json".to_string());
        let memory = Mutex::new(Vec::new());
        Self { path, memory }
    }

    pub fn append(&self, user: &str, bot: &str) {
        let mut mem = self.memory.lock().unwrap();
        mem.push(Dialogue {
            user: user.to_string(),
            bot: bot.to_string(),
        });
    }

    pub fn get_context_prompt(&self) -> String {
        let mem = self.memory.lock().unwrap();
        mem.iter()
            .map(|d| format!("user: {}\nbot: {}", d.user, d.bot))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn trim_oldest(&self, max_pairs: usize) {
        let mut mem = self.memory.lock().unwrap();
        if mem.len() > max_pairs {
            let excess = mem.len() - max_pairs;
            mem.drain(0..excess);
        }
    }

    pub fn save_to_file(&self) {
        let mem = self.memory.lock().unwrap();
        if let Ok(json) = serde_json::to_string_pretty(&*mem) {
            let _ = write(&self.path, json);
        }
    }

    pub fn load_from_file(&self) {
        if let Ok(text) = read_to_string(&self.path) {
            if let Ok(data) = serde_json::from_str::<Vec<Dialogue>>(&text) {
                let mut mem = self.memory.lock().unwrap();
                *mem = data;
            }
        }
    }

    pub fn reset(&self) {
        let mut mem = self.memory.lock().unwrap();
        mem.clear();
        let _ = write(&self.path, "[]");
    }
}