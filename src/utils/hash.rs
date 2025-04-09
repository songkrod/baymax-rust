use murmurhash3::murmurhash3_x64_128;
use std::string::String;

pub fn generate_item_id(data: &str) -> u64 {
    let hash = murmurhash3_x64_128(data.as_bytes(), 0); // ปรับใช้ seed ได้ตามต้องการ
    hash.0 // คืนค่า hash ที่เป็น 64-bit
}