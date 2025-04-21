use crate::services::vector_db::interface::SearchService;
use async_trait::async_trait;
use log::info;
use std::collections::HashMap;
use std::error::Error;
use std::sync::RwLock;

pub struct InMemorySearch {
    data: RwLock<HashMap<u64, Vec<f32>>>,
}

impl InMemorySearch {
    pub fn new() -> Self {
        InMemorySearch {
            data: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl SearchService for InMemorySearch {
    async fn search(&self, query_vector: Vec<f32>) -> Result<Vec<u64>, Box<dyn Error>> {
        let data = self.data.read().unwrap();

        let mut scored: Vec<(u64, f32)> = data
            .iter()
            .map(|(id, vec)| (*id, cosine_similarity(&query_vector, vec)))
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        Ok(scored.iter().take(5).map(|(id, _)| *id).collect())
    }

    async fn add_item(&self, item_id: u64, vector: Vec<f32>) -> Result<(), Box<dyn Error>> {
        let mut data = self.data.write().unwrap();
        data.insert(item_id, vector);
        Ok(())
    }

    async fn load_index(&self) -> Result<(), Box<dyn Error>> {
        info!("🧠 [InMemorySearch] ไม่มี index ให้โหลด — ใช้ fallback in-memory");
        Ok(())
    }
}

fn cosine_similarity(a: &Vec<f32>, b: &Vec<f32>) -> f32 {
    let dot: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let norm_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}