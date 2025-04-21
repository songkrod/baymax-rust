// 📁 services/search/google.rs

use std::error::Error;
use std::env;
use reqwest::Client;
use serde::Deserialize;
use crate::services::search::interface::{SearchResult, WebSearchService};
use async_trait::async_trait;

pub struct GoogleSearchService {
    api_key: String,
    cx: String,
    client: Client,
}

impl GoogleSearchService {
    pub fn new() -> Self {
        let api_key = env::var("google_api_key").expect("Missing GOOGLE_API_KEY");
        let cx = env::var("google_cse_id").expect("Missing GOOGLE_CSE_ID");

        Self {
            api_key,
            cx,
            client: Client::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct GoogleSearchResponse {
    items: Option<Vec<GoogleSearchItem>>,
}

#[derive(Debug, Deserialize)]
struct GoogleSearchItem {
    title: String,
    snippet: String,
    link: String,
}

#[async_trait]
impl WebSearchService for GoogleSearchService {
    async fn search(&self, query: &str) -> Result<SearchResult, Box<dyn Error + Send + Sync>> {
        let url = format!(
            "https://www.googleapis.com/customsearch/v1?q={query}&key={}&cx={}",
            self.api_key, self.cx
        );

        let res = self.client.get(&url).send().await?.json::<GoogleSearchResponse>().await?;

        if let Some(items) = res.items {
            if let Some(top) = items.first() {
                return Ok(SearchResult {
                    title: top.title.clone(),
                    snippet: top.snippet.clone(),
                    url: Some(top.link.clone()),
                });
            }
        }

        Err("No search results found".into())
    }
}