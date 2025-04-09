// ✅ src/memory/memory_manager.rs
use crate::services::search::manager::VectorSearch;
use crate::utils::embeddings::embed_text;
use crate::utils::hash::generate_item_id;
use std::collections::HashMap;
use serde_json::Value;
use log::{error, warn};
use strsim::jaro_winkler;

pub struct MemoryManager {
    vector_search: VectorSearch,
    memory: HashMap<String, Value>,
}

impl MemoryManager {
    pub fn new(vector_search: VectorSearch) -> Self {
        MemoryManager {
            vector_search,
            memory: HashMap::new(),
        }
    }

    pub async fn add_memory<T: serde::Serialize + ToString>(&mut self, key: &str, value: T) {
        let value_string = value.to_string();
        if let Ok(json) = serde_json::to_value(&value_string) {
            self.memory.insert(key.to_string(), json);
        }

        let item_id = generate_item_id(&value_string);
        let vector = embed_text(&value_string);

        if let Err(e) = self.vector_search.add_item(item_id, vector).await {
            error!("❌ Failed to add item to vector search: {}", e);
        }
    }

    /// ค้นหาความใกล้เคียง: ก่อนใช้ vector → ถ้าล้มเหลว → ใช้ fallback จากใน-memory
    pub async fn search_memory(&self, query: &str) -> Vec<String> {
        let vector = embed_text(query);
        match self.vector_search.search(vector).await {
            Ok(indices) if !indices.is_empty() => {
                // เอาแค่ ID ที่เจอ ไม่ได้ map กลับข้อความ (ยังไม่เชื่อม text storage)
                indices.iter().map(|id| format!("result_{}", id)).collect()
            }
            _ => {
                warn!("⚠️ [Fallback] ใช้ in-memory search แทน vector");
                self.search_memory_fallback(query)
            }
        }
    }

    /// ค้นหาแบบ manual จาก self.memory ถ้า vector ใช้ไม่ได้
    fn search_memory_fallback(&self, query: &str) -> Vec<String> {
        let mut results = vec![];

        for (_k, v) in &self.memory {
            if let Some(val) = v.as_str() {
                let score = jaro_winkler(query, val);
                if score > 0.85 {
                    results.push(format!("{} (score: {:.2})", val, score));
                }
            }
        }

        results
    }

    pub fn save_memory<T: serde::Serialize>(&mut self, key: &str, value: T) {
        if let Some(json) = serde_json::to_value(value).ok() {
            self.memory.insert(key.to_string(), json);
        }
    }

    pub fn recall_memory<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.memory.get(key).and_then(|value| {
            serde_json::from_value(value.clone()).ok()
        })
    }
}
