use crate::services::vector_db::interface::SearchService;
use crate::utils::file::ensure_file_exists;
use async_trait::async_trait;
use log::{info, error};
use std::error::Error;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

pub struct AnnoyPython {
    index_path: String,
}

impl AnnoyPython {
    /// สร้าง AnnoyPython โดยระบุ directory และนามสกุล (default = ann)
    pub fn new(index_dir: String, extension: Option<&str>) -> Self {
        let ext = extension.unwrap_or("ann");
        let index_path = format!("{}/index.{}", index_dir, ext);

        if let Err(e) = ensure_file_exists(&index_path) {
            error!("Failed to ensure index file: {}", e);
        }

        AnnoyPython { index_path }
    }

    /// คืนค่า path ที่ใช้จริง (เช่น สำหรับ debug หรือ export)
    pub fn get_path(&self) -> String {
        self.index_path.clone()
    }
}

#[async_trait]
impl SearchService for AnnoyPython {
    async fn search(&self, query_vector: Vec<f32>) -> Result<Vec<u64>, Box<dyn Error>> {
        let query_str = query_vector.iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
            .join(",");

        info!("🎧 [AnnoyPython] Searching for vector: {}", query_str);

        let mut child = Command::new("python3")
            .arg("scripts/vector_db.py")
            .arg("search")
            .arg(&self.index_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;  

        let stdin = child.stdin.as_mut().ok_or("Failed to open stdin")?;
        stdin.write_all(format!("{}\n", query_str).as_bytes())?;

        let stdout = child.stdout.take().ok_or("Failed to open stdout")?;
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let line = line?;
            info!("🐍 Python says: {}", line);

            if line.starts_with("ERROR") {
                error!("❌ AnnoyPython Error: {}", line);
                return Err(line.into());
            }

            if line.starts_with("RESULTS: ") {
                let result_str = line.trim_start_matches("RESULTS: ");
                let results_vec = result_str
                    .split(',')
                    .filter_map(|s| s.parse().ok())
                    .collect::<Vec<u64>>();
                return Ok(results_vec);
            }
        }

        Err("AnnoyPython: No results received.".into())
    }

    async fn add_item(&self, item_id: u64, vector: Vec<f32>) -> Result<(), Box<dyn Error>> {
        let vector_str = vector.iter()
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
            .join(",");

        info!("🎧 [AnnoyPython] Adding vector with item_id {}: {}", item_id, vector_str);

        let mut child = Command::new("python3")
            .arg("scripts/vector_db.py")
            .arg("add")
            .arg(&self.index_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;  

        let stdin = child.stdin.as_mut().ok_or("Failed to open stdin")?;
        stdin.write_all(format!("{}:{}\n", item_id, vector_str).as_bytes())?;

        let stdout = child.stdout.take().ok_or("Failed to open stdout")?;
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let line = line?;
            info!("🐍 Python says: {}", line);
        }

        Ok(())
    }

    async fn load_index(&self) -> Result<(), Box<dyn Error>> {
        info!("🎧 [AnnoyPython] Loading Annoy index from: {}", self.index_path);

        let mut child = Command::new("python3")
            .arg("scripts/vector_db.py")
            .arg("load")
            .arg(&self.index_path)
            .spawn()?;  

        let stdout = child.stdout.take().ok_or("Failed to open stdout")?;
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let line = line?;
            info!("🐍 Python says: {}", line);
        }

        Ok(())
    }
}