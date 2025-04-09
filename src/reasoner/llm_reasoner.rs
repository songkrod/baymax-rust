use crate::reasoner::template::build_reasoning_prompt;
use crate::reasoner::reasoning_result::ReasoningResult;
use crate::services::llm::manager::SmartLLM;
use crate::utils::{context, self_knowledge};
use log::{info, warn, debug};

use std::sync::Arc;

pub struct LLMReasoner {
    llm: Arc<SmartLLM>,
}

impl LLMReasoner {
    pub fn new(llm: Arc<SmartLLM>) -> Self {
        Self { llm }
    }

    pub async fn analyze(&self, agent_name: &str, user_text: &str, context: Option<&str>) -> ReasoningResult {
        info!("🧠 [LLMReasoner] วิเคราะห์ข้อความ: {}", user_text);
    
        let self_data = self_knowledge::load();
    
        let prompt = build_reasoning_prompt(
            agent_name,
            user_text,
            &self_data,
            context.unwrap_or(""),
        );
    
        match self.llm.complete(&prompt).await {
            Ok(response_text) => {
                // ตรวจสอบ response_text ก่อนที่จะแปลงเป็น JSON
                info!("🔍 [LLMResponse] Response text: {}", response_text);
        
                match ReasoningResult::from_json(&response_text) {
                    Ok(result) => result,
                    Err(e) => {
                        warn!("❌ Parse JSON ล้มเหลว: {}", e);
                        ReasoningResult::error("parse_failed", "ขออภัยครับ ผมยังวิเคราะห์ไม่สำเร็จ")
                    }
                }
            }
            Err(e) => {
                warn!("❌ เรียก GPT ล้มเหลว: {}", e);
                ReasoningResult::error("llm_failed", "เกิดปัญหาในการประมวลผลครับ")
            }
        }
    }
}