// 📁 services/search/google_scrape_python.rs

use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use std::error::Error;
use async_trait::async_trait;
use crate::services::search::interface::{SearchResult, WebSearchService};

pub struct GoogleScrapePython;

impl GoogleScrapePython {
    pub fn new() -> Self {
        GoogleScrapePython
    }
}

#[async_trait]
impl WebSearchService for GoogleScrapePython {
    async fn search(&self, query: &str) -> Result<SearchResult, Box<dyn Error + Send + Sync>> {
        let mut child = Command::new("python3")
            .arg("scripts/google_scrape.py")
            .arg(query)
            .stdout(Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
        let reader = BufReader::new(stdout);

        for line in reader.lines() {
            let line = line?;
            if line.starts_with("ERROR") {
                return Err(line.into());
            }
            if line.contains("⧙") {
                let parts: Vec<&str> = line.split("⧙").collect();
                if parts.len() == 2 {
                    return Ok(SearchResult {
                        title: parts[0].to_string(),
                        snippet: parts[1].to_string(),
                        url: None,
                    });
                }
            }
        }

        Err("No valid result returned from python scraper".into())
    }
}