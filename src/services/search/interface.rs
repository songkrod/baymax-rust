// src/services/search/interface.rs

use async_trait::async_trait;
use std::error::Error;

/// Trait สำหรับระบบค้นหาคล้ายคลึง (Search)
#[async_trait]
pub trait SearchService: Send + Sync {
    /// ฟังก์ชันสำหรับการค้นหาคล้ายคลึง โดยรับ query vector
    async fn search(&self, query_vector: Vec<f32>) -> Result<Vec<u64>, Box<dyn Error>>;

    /// ฟังก์ชันสำหรับการเพิ่มข้อมูลไปยัง index
    async fn add_item(&self, item_id: u64, vector: Vec<f32>) -> Result<(), Box<dyn Error>>;

    /// ฟังก์ชันสำหรับโหลด index
    async fn load_index(&self) -> Result<(), Box<dyn Error>>;
}
