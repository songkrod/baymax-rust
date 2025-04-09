// 🔒 คำพูดหลอน (hallucinations) ที่มักจะถูกรับรู้ผิดจาก ASR
pub fn is_hallucination(text: &str) -> bool {
    let text = text.to_lowercase();
    let patterns = [
        "บรรยายธรรมอิสลามครูแอ",
        "โปรดติดตามตอนต่อไป",
        "รักเด็ก",
        "ขอบคุณที่รับชม",
        "thanks for watching",
        "please like and subscribe",
        "stay tuned for more",
        "have a great day",
        "ご視聴ありがとうございました",
        "次回をお楽しみに",
        "いいねとチャンネル登録お願いします",
        "시청해주셔서 감사합니다",
        "좋아요와 구독 부탁드려요",
    ];
    patterns.iter().any(|&phrase| text.contains(phrase))
}
