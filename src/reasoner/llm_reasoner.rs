use crate::reasoner::template::build_reasoning_prompt;
use crate::reasoner::reasoning_result::ReasoningResult;
use crate::services::llm::manager::SmartLLM;
use crate::utils::self_knowledge;
use log::{info, warn};
use std::sync::Arc;
use serde::Deserialize;
use crate::reasoner::template::build_insight_prompt;

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
                        ReasoningResult::error("parse_failed")
                    }
                }
            }
            Err(e) => {
                warn!("❌ เรียก GPT ล้มเหลว: {}", e);
                ReasoningResult::error("llm_failed")
            }
        }
    }

    pub async fn analyze_insight(&self, input: &str) -> Option<ReasoningResult> {
        #[derive(Debug, Deserialize)]
        struct RawInsight {
            intent: String,
            emotion: String,
            confidence: f32,
        }

        let prompt = build_insight_prompt(input);

        match self.llm.complete(&prompt).await {
            Ok(mut raw) => {
                // strip ```json or ``` if present
                raw = raw.trim().trim_start_matches("```json").trim_start_matches("```").trim_end_matches("```").trim().to_string();

                match serde_json::from_str::<RawInsight>(&raw) {
                    Ok(parsed) => Some(ReasoningResult {
                        intent: parsed.intent,
                        emotion: parsed.emotion,
                        confidence: Some(parsed.confidence),
                        action: None,
                        hardware_required: None,
                    }),
                    Err(e) => {
                        warn!("❌ วิเคราะห์ Insight แล้ว parse ไม่ได้: {} | raw: {}", e, raw);
                        None
                    }
                }
            }
            Err(e) => {
                warn!("❌ วิเคราะห์ Insight แล้ว LLM พัง: {}", e);
                None
            }
        }
    }
}
