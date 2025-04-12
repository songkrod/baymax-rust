/// ตรวจว่าตัวอักษรเป็นตัวจบคำในภาษาไทยหรือไม่
pub fn is_thai_boundary(c: char) -> bool {
    matches!(c, ' ' | 'ๆ' | '.' | '?' | '!' | ',' | ':' | ';') || is_thai_vowel_or_tone(c)
}

/// ตรวจว่าสระหรือวรรณยุกต์ไทย
pub fn is_thai_vowel_or_tone(c: char) -> bool {
    matches!(c,
        'ะ' | 'า' | 'ิ' | 'ี' | 'ึ' | 'ื' |
        'ุ' | 'ู' | 'เ' | 'แ' | 'โ' | 'ใ' | 'ไ' |
        '่' | '้' | '๊' | '๋' | '์')
}

/// หาตำแหน่งที่ควรตัดคำใน buffer เพื่อส่งเข้า TTS
pub fn find_cut_position(buffer: &str, min_len: usize) -> Option<usize> {
    let chars: Vec<char> = buffer.chars().collect();

    for i in (min_len..chars.len()).rev() {
        if is_thai_boundary(chars[i]) {
            return Some(i + 1); // รวมตัว boundary
        }
    }

    None
}

/// แยก buffer เป็น chunk ละ N คำ (ใช้กับ GPT streaming แบบมี space)
pub fn split_to_word_chunks(text: &str, word_limit: usize) -> (Vec<String>, String) {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut chunks = Vec::new();
    let mut i = 0;

    while i + word_limit <= words.len() {
        let chunk = words[i..i + word_limit].join(" ");
        chunks.push(chunk);
        i += word_limit;
    }

    let leftover = words[i..].join(" ");
    (chunks, leftover)
}

/// ชนิดของ chunk ที่อาจจะมี pause หรือไม่
#[derive(Debug)]
pub enum SmartChunk {
    Normal(String),      // ⧙ → ไม่หยุดพูด
    WithPause(String),   // ※ → หยุดพูด
}

/// ใช้กับภาษาไทย: แบ่งวรรคแบบฉลาดตามเครื่องหมายพิเศษที่ GPT แทรกเข้ามา
/// ⧙ = split เฉย ๆ, พูดต่อเนื่องกันได้เลย
/// ※ = pause ก่อนพูดประโยคถัดไป
pub fn split_smart_thai_chunks(text: &str, _min_len: usize) -> (Vec<SmartChunk>, String) {
    use log::debug;
    let mut chunks = vec![];
    let mut current = String::new();
    debug!("🧠 chunker ได้รับข้อความ: '{}'", text);

    for c in text.chars() {
        match c {
            '※' => {
                if !current.trim().is_empty() {
                    let chunk = current.trim().to_string();
                    debug!("🧠 ตัด chunk (※): '{}'", chunk);
                    chunks.push(SmartChunk::WithPause(chunk));
                    current.clear();
                }
            },
            '⧙' => {
                if !current.trim().is_empty() {
                    let chunk = current.trim().to_string();
                    debug!("🧠 ตัด chunk (⧙): '{}'", chunk);
                    chunks.push(SmartChunk::Normal(chunk));
                    current.clear();
                }
            },
            _ => current.push(c),
        }
    }

    (chunks, current)
}