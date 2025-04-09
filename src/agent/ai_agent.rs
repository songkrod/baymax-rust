use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use log::{error, info, warn};
use tokio::time::{sleep, Duration};

use crate::reasoner::llm_reasoner::LLMReasoner;
use crate::reasoner::reasoning_result::ReasoningResult;
use crate::services::asr::manager::SmartASR;
use crate::services::llm::manager::SmartLLM;
use crate::services::tts::manager::SmartTTS;
use crate::utils::hallucination::is_hallucination;

/// Async Skill ฟังก์ชันที่สามารถเรียกได้ภายหลัง เช่น "say", "rest"
type SkillFn = fn(Option<&str>) -> Pin<Box<dyn Future<Output = ()> + Send>>;

/// พาธไฟล์เสียงที่ใช้ในการบันทึกและแปลง
const RAW_AUDIO_PATH: &str = "src/data/caches/audio/input.wav";

/// ตัวแทนของ Agent ที่สามารถพูด, ฟัง, คิด, และเรียนรู้ skill ได้
pub struct AiAgent {
    pub name: String,
    pub tts: Arc<SmartTTS>,
    pub asr: Arc<SmartASR>,
    pub llm: Arc<SmartLLM>,
    pub reasoner: Arc<LLMReasoner>,
    pub skills: HashMap<String, SkillFn>,
}

impl AiAgent {
    pub fn new(name: &str) -> Self {
        info!("🤖 Initializing AiAgent with name: {}", name);

        let llm = Arc::new(SmartLLM::new());
        let reasoner = Arc::new(LLMReasoner::new(Arc::clone(&llm)));

        info!("🧠 Reasoner initialized");
        info!("🗣️ TTS engine ready");
        info!("🧏 ASR engine ready");

        Self {
            name: name.to_string(),
            tts: Arc::new(SmartTTS::new()),
            asr: Arc::new(SmartASR::new()),
            llm,
            reasoner,
            skills: HashMap::new(),
        }
    }

    pub async fn say(&self, msg: &str) {
        info!("🎙️ {} กำลังพูด: {}", self.name, msg);
        if let Err(e) = self.tts.speak(msg).await {
            error!("❌ พูดไม่สำเร็จ: {}", e);
            self.say_error(&format!("พูดไม่ได้: {}", e)).await;
        }
    }

    pub fn is_speaking(&self) -> bool {
        self.tts.is_speaking()
    }

    pub async fn do_skill(&self, name: &str, args: Option<&str>) {
        info!("🛠️ เรียกใช้ skill: '{}' args: {:?}", name, args);
        match name {
            "say" => {
                if let Some(text) = args {
                    self.say(text).await;
                } else {
                    self.fallback().await;
                }
            }
            other => {
                if let Some(skill_fn) = self.skills.get(other) {
                    skill_fn(args).await;
                } else {
                    warn!("⚠️ Skill '{}' ไม่พบในระบบ", other);
                    self.fallback().await;
                }
            }
        }
    }

    pub fn learn_skill(&mut self, name: &str, function: SkillFn) {
        info!("🧠 {} เรียนรู้ skill ใหม่: '{}'", self.name, name);
        self.skills.insert(name.to_string(), function);
    }

    pub async fn listen(&self) -> Option<String> {
        while self.tts.is_speaking() {
            sleep(Duration::from_millis(300)).await;
        }

        info!("🎤 [{}] เริ่มบันทึกเสียงผู้ใช้...", self.name);

        let result = self
            .asr
            .listen(RAW_AUDIO_PATH, Some(10_000), Some(1000))
            .await;

        match &result {
            Some(text) => {
                info!("🧠 [{}] ผู้ใช้พูดว่า: {}", self.name, text);
                if self.is_hallucination(text) {
                    warn!("🌀 ตรวจพบข้อความหลอน: {}", text);
                    return None;
                }
            }
            None => {
                warn!("📭 [{}] ไม่พบข้อความเสียง", self.name);
            }
        }

        result
    }

    pub async fn reason(&self, input: &str, context: Option<&str>) -> ReasoningResult {
        info!("🧠 [{}] กำลังคิดคำตอบจากข้อความ: {}", self.name, input);
        self.reasoner.analyze(&self.name, input, context).await
    }

    async fn say_error(&self, msg: &str) {
        error!("❌ {} error: {}", self.name, msg);
        let polite_msg = format!("ขออภัยครับ เกิดข้อผิดพลาด: {}", msg);
        let _ = self.tts.speak(&polite_msg).await;
    }

    fn is_hallucination(&self, text: &str) -> bool {
        let result = is_hallucination(text);
        if result {
            warn!("🌀 ตรวจพบข้อความหลอน: {}", text);
        }
        result
    }

    pub async fn fallback(&self) {
        warn!("🤔 {} ไม่เข้าใจคำสั่งที่ได้รับ", self.name);
        self.say("ขออภัยครับ ผมยังไม่เข้าใจคำสั่งนั้น").await;
    }
}
