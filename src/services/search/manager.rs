// 📁 services/search/manager.rs

use std::sync::Arc;
use log::{info, warn, error};

use super::interface::{WebSearchService, SearchResult};
use super::google_api::GoogleSearchService;
// use super::google_scraper::GoogleScraperService;
use super::google_scraper_python::GoogleScrapePython;
// use super::serper::SerperSearchService; // optional future fallback

/// ระบบค้นหาข้อมูลจากเว็บแบบ pluggable backend (เช่น Google API, scraping)
pub struct SmartSearch {
    primary: Arc<dyn WebSearchService + Send + Sync>,
    fallback: Option<Arc<dyn WebSearchService + Send + Sync>>, // optional fallback
}

impl SmartSearch {
    pub fn new() -> Self {
        let backend = std::env::var("search_backend").unwrap_or_else(|_| "google_api".to_string());

        let primary: Arc<dyn WebSearchService + Send + Sync> = match backend.as_str() {
            "google_api" => {
                info!("🔍 [Search] ใช้งาน Google Search API เป็นระบบหลัก");
                Arc::new(GoogleSearchService::new())
            }
            // "google_scrape" => {
            //     info!("🔍 [Search] ใช้งาน Google Scraper เป็นระบบหลัก");
            //     Arc::new(GoogleScraperService::new())
            // }
            "google_scrape_py" => {
                info!("🔍 [Search] ใช้งาน Google Scraper (Python) เป็นระบบหลัก");
                Arc::new(GoogleScrapePython::new())
            }
            // "serper" => {
            //     info!("🔍 [Search] ใช้งาน Serper เป็นระบบหลัก");
            //     Arc::new(SerperSearchService::new())
            // }
            unknown => {
                warn!("❗ [Search] ค่า backend '{}' ไม่รู้จัก → fallback เป็น Google API", unknown);
                Arc::new(GoogleSearchService::new())
            }
        };

        Self {
            primary,
            fallback: None, // fallback ยังไม่เปิดใช้งาน
        }
    }

    /// ค้นหาข้อมูลตามคำถามของผู้ใช้
    pub async fn search(&self, query: &str) -> Result<SearchResult, Box<dyn std::error::Error + Send + Sync>> {
        info!("🔎 [Search] เริ่มค้นหาข้อมูลจากคำถาม: {}", query);

        match self.primary.search(query).await {
            Ok(result) => {
                info!("✅ [Search] พบข้อมูล: {}", result.title);
                Ok(result)
            }
            Err(e) => {
                warn!("⚠️ [Search] ล้มเหลวจาก primary: {}", e);
                if let Some(fallback) = &self.fallback {
                    fallback.search(query).await
                } else {
                    Err(e)
                }
            }
        }
    }
}
