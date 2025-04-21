use crate::services::vector_db::annoy_python::AnnoyPython;
use crate::services::vector_db::in_memory::InMemorySearch;
use crate::services::vector_db::interface::SearchService;
use log::{info, warn};
use std::error::Error;
use std::sync::Arc;

const INDEX_DIR: &str = "src/data/memory";

#[derive(Clone)]
pub struct VectorSearch {
    primary: Arc<dyn SearchService + Send + Sync>,
    fallback: Arc<dyn SearchService + Send + Sync>,
}

impl VectorSearch {
    pub fn new() -> Self {
        let backend = std::env::var("vector_db_backend").unwrap_or_else(|_| "annoy_python".to_string());

        let primary: Arc<dyn SearchService + Send + Sync> = match backend.as_str() {
            "annoy_python" => {
                info!("🧠 [Search] ใช้งาน AnnoyPython เป็นระบบหลัก");
                Arc::new(AnnoyPython::new(INDEX_DIR.to_string(), Some("ann")))
            }
            unknown => {
                warn!("❗ [Search] backend '{}' ไม่รู้จัก → fallback เป็น Annoy", unknown);
                Arc::new(AnnoyPython::new(INDEX_DIR.to_string(), Some("ann")))
            }
        };

        let fallback: Arc<dyn SearchService + Send + Sync> = Arc::new(InMemorySearch::new());

        Self { primary, fallback }
    }

    pub async fn search(&self, query_vector: Vec<f32>) -> Result<Vec<u64>, Box<dyn Error>> {
        match self.primary.search(query_vector.clone()).await {
            Ok(result) => Ok(result),
            Err(e) => {
                warn!("⚠️ [Primary] ล้มเหลว: {} → fallback เป็น in-memory", e);
                self.fallback.search(query_vector).await
            }
        }
    }

    pub async fn add_item(&self, item_id: u64, vector: Vec<f32>) -> Result<(), Box<dyn Error>> {
        self.fallback.add_item(item_id, vector.clone()).await?;
        self.primary.add_item(item_id, vector).await
    }

    pub async fn load_index(&self) -> Result<(), Box<dyn Error>> {
        self.primary.load_index().await
    }
}