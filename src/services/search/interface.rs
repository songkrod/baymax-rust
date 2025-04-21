// 📁 services/search/interface.rs

use async_trait::async_trait;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub title: String,
    pub snippet: String,
    pub url: Option<String>,
}

#[async_trait]
pub trait WebSearchService: Send + Sync {
    async fn search(&self, query: &str) -> Result<SearchResult, Box<dyn Error + Send + Sync>>;
}
