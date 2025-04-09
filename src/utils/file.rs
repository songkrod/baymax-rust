use std::fs::{create_dir_all, File};
use std::path::Path;
use std::io::Write;
use std::error::Error;

pub fn ensure_file_exists(file_path: &str) -> Result<(), Box<dyn Error>> {
    let path = Path::new(file_path);

    if !path.exists() {
        // ถ้าไม่พบไฟล์ เราจะสร้างมัน
        if path.is_dir() {
            return Err(format!("Expected file but found directory: {}", file_path).into());
        }
        
        // ถ้าไฟล์ไม่พบ เราจะสร้างไฟล์ใหม่
        if let Some(parent) = path.parent() {
            // สร้างโฟลเดอร์ถ้ายังไม่มี
            create_dir_all(parent)?;
        }

        // สร้างไฟล์ใหม่
        let mut file = File::create(file_path)?;
        file.write_all(b"")?;  // สร้างไฟล์ว่าง
        println!("File created: {}", file_path);
    }

    Ok(())
}