use std::fs;
use std::path::Path;
use std::env;
use serde::{Deserialize, Serialize};
use log::{info, warn};

use crate::perception::name::name_utils::{normalize, find_best_match};

const MEMORY_PATH: &str = env::var("name_memory_path").unwrap_or_else(|_| "src/data/memory/name_memory.json".to_string());
const MATCH_THRESHOLD: f64 = 0.88;

#[derive(Debug, Serialize, Deserialize)]
pub struct NameMemory {
    pub names: Vec<String>,
}

impl NameMemory {
    pub fn load() -> Self {
        if Path::new(MEMORY_PATH).exists() {
            let data = fs::read_to_string(MEMORY_PATH).unwrap_or_default();
            serde_json::from_str(&data).unwrap_or_else(|_| NameMemory { names: vec![] })
        } else {
            NameMemory { names: vec![] }
        }
    }

    pub fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            fs::write(MEMORY_PATH, json).unwrap_or_else(|e| {
                warn!("❌ ไม่สามารถบันทึกชื่อลงไฟล์ได้: {}", e);
            });
        }
    }

    pub fn add_name(&mut self, name: &str) {
        if !self.names.iter().any(|n| normalize(n) == normalize(name)) {
            info!("📌 เพิ่มชื่อใหม่: {}", name);
            self.names.push(name.to_string());
            self.save();
        }
    }
}

/// ตรวจว่ามีการเรียกชื่อหุ่นหรือไม่
pub fn is_called_by_name(transcript: &str) -> Option<String> {
    let memory = NameMemory::load();
    if memory.names.is_empty() {
        warn!("⚠️ ยังไม่มีชื่อที่จดจำไว้เลย");
        return None;
    }

    match find_best_match(transcript, &memory.names, MATCH_THRESHOLD) {
        Some((matched, score, word)) => {
            info!("✅ ตรวจพบการเรียกชื่อ '{}' (match: '{}', {:.2})", word, matched, score);
            Some(matched.to_string())
        }
        None => {
            info!("🫥 ไม่พบคำที่ใกล้เคียงชื่อหุ่นใน transcript: {}", transcript);
            None
        }
    }
}