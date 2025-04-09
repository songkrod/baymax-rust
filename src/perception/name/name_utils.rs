use strsim::jaro_winkler;

/// แปลงข้อความให้เป็นรูปแบบที่ใช้เปรียบเทียบง่ายขึ้น
pub fn normalize(text: &str) -> String {
    text.to_lowercase().replace(' ', "")
}

/// แยกคำภาษาไทยด้วย heuristic ง่าย ๆ — ตัดคำใหม่เมื่อเจอพยัญชนะต้นคำ
pub fn naive_split_thai(text: &str) -> Vec<String> {
    let mut result = vec![];
    let mut current = String::new();
    for c in text.chars() {
        if is_thai_consonant(c) && !current.is_empty() {
            result.push(current.clone());
            current.clear();
        }
        current.push(c);
    }
    if !current.is_empty() {
        result.push(current);
    }
    result
}

/// ตรวจว่าตัวอักษรเป็นพยัญชนะไทยหรือไม่ (ก-ฮ)
pub fn is_thai_consonant(c: char) -> bool {
    ('ก'..='ฮ').contains(&c)
}

/// ตรวจว่าคำไหนในประโยคคล้ายชื่อหุ่นมากที่สุด
/// คืน (ชื่อที่ match, คะแนนความคล้าย, คำที่เรียก)
pub fn find_best_match<'a>(
    transcript: &'a str,
    known_names: &'a [String],
    threshold: f64,
) -> Option<(&'a str, f64, String)> {
    let normalized_transcript = normalize(transcript);

    // เช็ก exact match แบบ normalize
    for name in known_names {
        if normalized_transcript.contains(&normalize(name)) {
            return Some((name, 1.0, name.clone()));
        }
    }

    let words = naive_split_thai(transcript);

    let mut best_score = 0.0;
    let mut best_word = "";
    let mut matched_name = "";

    for word in &words {
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