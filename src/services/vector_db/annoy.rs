use crate::services::vector_db::interface::SearchService;
use std::sync::Arc; // ใช้ Arc แทน Rc
use annoy_rs::{AnnoyIndex, IndexType};
use async_trait::async_trait;

pub struct AnnoySearch {
    index: Arc<AnnoyIndex>, // เปลี่ยนเป็น Arc
}

impl AnnoySearch {
    pub fn new(dim: usize) -> Self {
        // ใช้ Arc สำหรับแชร์ข้อมูลในหลาย threads
        let index = Arc::new(AnnoyIndex::load(dim, "data/memory/index.ann", IndexType::Angular)
            .expect("Failed to load AnnoyIndex"));
        
        AnnoySearch { index }
    }

    pub fn add_item(&mut self, id: usize, vector: Vec<f32>) {
        // ใช้ add_item
        self.index.add_item(id as i32, &vector);
    }

    pub fn build(&mut self) {
        // ใช้ build
        self.index.build(10);
    }
}

#[async_trait]
impl SearchService for AnnoySearch {
    async fn search(&self, query: Vec<f32>, num_neighbors: usize) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
        // ใช้ get_item_vector แทน get_nns_by_vector
        let ids = self.index.get_item_vector(query, num_neighbors);
        Ok(ids.into_iter().map(|id| id as usize).collect())
    }
}