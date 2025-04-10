use async_trait::async_trait;

#[async_trait]
pub trait TTSService: Send + Sync {
    async fn speak(&self, text: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// ใช้สำหรับกรณีที่ GPT ตอบกลับมาเป็นข้อความยาว ๆ ทีละบรรทัด
    async fn stream_speak(&self, _chunks: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        for chunk in _chunks {
            self.speak(&chunk)
                .await
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e });
        }
        Ok(())
    }

    async fn synthesize(&self, text: &str) -> Result<Vec<u8>, String>;
}