use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;

use log::{debug, error, info, warn};
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use crate::services::asr::manager::SmartASR;
use crate::services::llm::manager::SmartLLM;
use crate::services::tts::queue::TTSQueue;
use crate::utils::hallucination::is_hallucination;
use crate::utils::streaming::chunker::{split_smart_thai_chunks, SmartChunk};
use crate::reasoner::llm_reasoner::LLMReasoner;
use crate::reasoner::reasoning_result::ReasoningResult;
use crate::reasoner::template::build_streaming_prompt;

type SkillFn = fn(Option<&str>) -> Pin<Box<dyn Future<Output = ()> + Send>>;

const RAW_AUDIO_PATH: &str = "src/data/caches/audio/input.wav";

pub struct AiAgent {
    pub name: String,
    pub tts: Arc<TTSQueue>,
    pub asr: Arc<SmartASR>,
    pub llm: Arc<SmartLLM>,
    pub reasoner: Arc<LLMReasoner>,
    pub skills: HashMap<String, SkillFn>,
    is_speaking_flag: Arc<Mutex<bool>>,
}

impl AiAgent {
    pub fn new(name: &str, tts: Arc<TTSQueue>) -> Self {
        info!("🤖 Initializing AiAgent with name: {}", name);

        let llm = Arc::new(SmartLLM::new());
        let reasoner = Arc::new(LLMReasoner::new(Arc::clone(&llm)));

        info!("🫠 Reasoner initialized");
        info!("🔡 TTS engine ready");
        info!("🫯 ASR engine ready");

        Self {
            name: name.to_string(),
            tts,
            asr: Arc::new(SmartASR::new()),
            llm,
            reasoner,
            skills: HashMap::new(),
            is_speaking_flag: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn say(&self, msg: &str) {
        {
            let mut flag = self.is_speaking_flag.lock().await;
            *flag = true;
        }

        info!("🎡 {} กำลังพูด: {}", self.name, msg);
        self.tts.enqueue_and_wait(msg).await;
        self.await_speaking_done().await;

        {
            let mut flag = self.is_speaking_flag.lock().await;
            *flag = false;
        }
    }

    pub async fn await_speaking_done(&self) {
        while self.tts.is_speaking().await {
            sleep(Duration::from_millis(100)).await;
        }
    }

    pub async fn think_and_say_streaming(&self, input: &str) -> String {
        info!("🚦 เรียกใช้ think_and_say_streaming แล้ว");
        let name = self.name.clone();
        let tts = Arc::clone(&self.tts);
        let full_reply = Arc::new(Mutex::new(String::new()));
        let speaking_flag = Arc::clone(&self.is_speaking_flag);
        let t0 = Instant::now();

        {
            let mut flag = speaking_flag.lock().await;
            *flag = true;
        }

        info!("💬 [{}] เริ่มตอบแบบ streaming: {}", name, input);

        self.tts.enqueue_with_start_time("ขอผมตรวจสอบสักครู่นะครับ", Some(t0));

        let buffer = Arc::new(Mutex::new(String::new()));
        let reply_for_closure = Arc::clone(&full_reply);

        let streaming_prompt = build_streaming_prompt(input);
        debug!("📋 [Prompt] {}", streaming_prompt);

        debug!("🧪 [LLM] เริ่ม stream_reply");
        let result = self.llm.stream_reply(&streaming_prompt, {
            let buffer = Arc::clone(&buffer);
            let tts = Arc::clone(&tts);

            move |chunk| {
                debug!("🧩 [Chunk] ได้รับ: '{}'", chunk);

                let buffer = Arc::clone(&buffer);
                let tts = Arc::clone(&tts);
                let reply_for_closure = Arc::clone(&reply_for_closure);
                let t0 = t0.clone();

                tokio::spawn(async move {
                    reply_for_closure.lock().await.push_str(&chunk);

                    let mut buf = buffer.lock().await;
                    buf.push_str(&chunk);
                    debug!("🧠 [Buffer] ปัจจุบัน: {}", buf);

                    let (chunks, rest) = split_smart_thai_chunks(&buf, 0);
                    *buf = rest;

                    for chunk in chunks {
                        match chunk {
                            SmartChunk::Normal(text) => {
                                debug!("📤 [TTS] ส่งเข้า TTS (ระหว่าง stream): {}", text);
                                tts.enqueue_with_start_time(&text, Some(t0));
                            }
                        }
                    }
                });
            }
        }).await;

        if result.is_ok() {
            let mut buf = buffer.lock().await;
            let leftover = buf.trim();
            if !leftover.is_empty() {
                debug!("🔚 [Flush] ส่ง chunk ปิดท้ายเข้า TTS: {}", leftover);
                tts.enqueue_with_start_time(leftover, Some(t0));
                buf.clear();
            }
        } else {
            error!("🛑 stream_reply ล้มเหลว: {:?}", result);
            self.tts.enqueue("ขออภัยครับ ผมตอบไม่ได้ในตอนนี้");
        }

        self.await_speaking_done().await;

        {
            let mut flag = speaking_flag.lock().await;
            *flag = false;
        }

        let reply = {
            let lock = full_reply.lock().await;
            lock.clone()
        };

        info!("🧾 GPT full reply: {}", reply);
        reply
    }

    pub async fn is_speaking(&self) -> bool {
        *self.is_speaking_flag.lock().await
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
        info!("🫠 {} เรียนรู้ skill ใหม่: '{}'", self.name, name);
        self.skills.insert(name.to_string(), function);
    }

    pub async fn listen(&self) -> Option<String> {
        self.await_speaking_done().await;

        self.tts.play_beep_start().await;
        info!("🎤 [{}] เริ่มบันทึกเสียงผู้ใช้...", self.name);

        let result = self
            .asr
            .listen(RAW_AUDIO_PATH, Some(10_000), Some(1000))
            .await;

        self.tts.play_beep_end().await;

        match &result {
            Some(text) => {
                info!("🧠 [{}] ผู้ใช้พูดว่า: {}", self.name, text);
                if self.is_hallucination(text) {
                    warn!("🌀 ตรวจพบข้อความหลอน: {}", text);
                    return None;
                }
            }
            None => {
                warn!("📬 [{}] ไม่พบข้อความเสียง", self.name);
            }
        }

        result
    }

    pub async fn reason(&self, input: &str, context: Option<&str>) -> ReasoningResult {
        info!("🫠 [{}] กำลังคิดคำตอบจากข้อความว่า: {}", self.name, input);
        self.reasoner.analyze(&self.name, input, context).await
    }

    async fn say_error(&self, msg: &str) {
        error!("❌ {} error: {}", self.name, msg);
        let polite_msg = format!("ขออภัยครับ เกิดข้อผิดพลาด: {}", msg);
        self.tts.enqueue(&polite_msg);
    }

    fn is_hallucination(&self, text: &str) -> bool {
        let result = is_hallucination(text);
        if result {
            warn!("🌀 ตรวจพบข้อความหลอน: {}", text);
        }
        result
    }

    pub async fn fallback(&self) {
        warn!("🧐 {} ไม่เข้าใจคำสั่งที่ได้รับ", self.name);
        self.say("ขออภัยครับ ผมยังไม่เข้าใจคำสั่งนั้น").await;
    }
}
