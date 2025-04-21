// 📁 services/search/google_scraper.rs

use std::error::Error;
use async_trait::async_trait;
use reqwest::Client;
use scraper::{Html, Selector};
use crate::services::search::interface::{SearchResult, WebSearchService};

pub struct GoogleScraperService {
    client: Client,
}

impl GoogleScraperService {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (compatible; BaymaxBot/1.0; +http://example.com)")
                .build()
                .expect("Failed to build HTTP client"),
        }
    }
}

#[async_trait]
impl WebSearchService for GoogleScraperService {
    async fn search(&self, query: &str) -> Result<SearchResult, Box<dyn Error + Send + Sync>> {
        let url = format!("https://www.google.com/search?q={}", query);
        let res = self.client.get(&url).send().await?.text().await?;

        let document = Html::parse_document(&res);
        let selector = Selector::parse("div.BNeawe.vvjwJb.AP7Wnd").unwrap();
        let snippet_selector = Selector::parse("div.BNeawe.s3v9rd.AP7Wnd").unwrap();

        let title = document.select(&selector).next()
            .map(|e| e.inner_html())
            .unwrap_or_else(|| "ไม่พบชื่อเรื่อง".to_string());

        let snippet = document.select(&snippet_selector).next()
            .map(|e| e.inner_html())
            .unwrap_or_else(|| "ไม่พบคำอธิบาย".to_string());

        Ok(SearchResult {
            title,
            snippet,
            url: None, // scraping ไม่ได้ลิงก์โดยตรง
        })
    }
}