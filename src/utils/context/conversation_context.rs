// src/utils/context/conversation_context.rs

use std::fs;
use std::path::Path;
use std::sync::Mutex;
use lazy_static::lazy_static;
use log::warn;

const CONTEXT_FILE: &str = "src/data/memory/context_cache.json";

lazy_static! {
    static ref CONVERSATION_CONTEXT: Mutex<ConversationContext> = Mutex::new(ConversationContext::load());
}

#[derive(Debug, Clone)]
pub struct ConversationContext {
    pub log: Vec<String>, // เก็บ user/bot ทีละบรรทัด
    pub max_turns: usize, // จำกัดแค่ N บรรทัดล่าสุด
}

impl ConversationContext {
    pub fn new(max_turns: usize) -> Self {
        Self {
            log: Vec::new(),
            max_turns,
        }
    }

    pub fn load() -> Self {
        if Path::new(CONTEXT_FILE).exists() {
            match fs::read_to_string(CONTEXT_FILE) {
                Ok(data) => {
                    let log: Vec<String> = data.lines().map(|s| s.to_string()).collect();
                    Self { log, max_turns: 10 }
                }
                Err(e) => {
                    warn!("โหลด context ไม่สำเร็จ: {}", e);
                    Self::new(10)
                }
            }
        } else {
            Self::new(10)
        }
    }

    pub fn save(&self) {
        if let Err(e) = fs::write(CONTEXT_FILE, self.log.join("\n")) {
            warn!("บันทึก context ไม่สำเร็จ: {}", e);
        }
    }

    pub fn add_turn(&mut self, who: &str, message: &str) {
        self.log.push(format!("{}: {}", who, message));
        if self.log.len() > self.max_turns {
            self.log.drain(0..(self.log.len() - self.max_turns));
        }
        self.save();
    }

    pub fn get_recent(&self) -> String {
        self.log.join("\n")
    }

    pub fn reset(&mut self) {
        self.log.clear();
        self.save();
    }

    // ใช้เรียกจากภายนอก
    pub fn global() -> std::sync::MutexGuard<'static, ConversationContext> {
        CONVERSATION_CONTEXT.lock().unwrap()
    }
}
