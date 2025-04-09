use strsim::jaro_winkler;

/// แปลงข้อความให้เป็นรูปแบบที่ใช้เปรียบเทียบง่ายขึ้น
/// - ตัวเล็กทั้งหมด
/// - ตัดช่องว่าง
pub fn normalize(text: &str) -> String {
    text.to_lowercase().replace(' ', "")
}

/// ตรวจว่าคำไหนในประโยคคล้ายชื่อหุ่นมากที่สุด
/// คืนมาเป็น (ชื่อที่ใกล้เคียง, คะแนนความคล้าย, คำที่ใช้เรียก)
pub fn find_best_match<'a>(
    transcript: &'a str,
    known_names: &'a [String],
    threshold: f64,
) -> Option<(&'a str, f64, String)> {
    let norm_transcript = normalize(transcript);
    let words: Vec<&str> = transcript.split_whitespace().collect();

    let mut best_score = 0.0;
    let mut best_word = "";
    let mut matched_name = "";

    for word in words {
        let word_norm = normalize(word);
        for name in known_names {
            let score = jaro_winkler(&word_norm, &normalize(name));
            if score > best_score && score >= threshold {
                best_score = score;
                best_word = word;
                matched_name = name;
            }
        }
    }

    if best_score >= threshold {
        Some((matched_name, best_score, best_word.to_string()))
    } else {
        None
    }
}