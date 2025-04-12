use async_trait::async_trait;

#[async_trait]
pub trait SpeakerBackend: Send + Sync {
    async fn play(&self, data: &[u8]) -> Result<(), String>;
    async fn is_busy(&self) -> bool;

    /// 🔊 ติ๊ดเบาๆ เมื่อเริ่มฟัง
    async fn play_beep_start(&self);

    /// 🔊 ติ๊ดๆ เบาๆ เมื่อหยุดฟัง
    async fn play_beep_end(&self);
}