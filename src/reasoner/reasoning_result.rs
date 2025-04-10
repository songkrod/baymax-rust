// 📁 src/reasoner/reasoning_result.rs

use serde::{Deserialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ReasoningResult {
    pub intent: String,
    pub emotion: String,

    #[serde(default)]
    pub action: Option<String>,

    #[serde(default)]
    pub hardware_required: Option<String>,

    #[serde(default)]
    pub confidence: Option<f32>,
}

impl ReasoningResult {
    pub fn error(intent: &str) -> Self {
        Self {
            intent: intent.to_string(),
            emotion: "neutral".to_string(),
            action: None,
            hardware_required: None,
            confidence: None,
        }
    }

    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    pub fn new(
        intent: &str,
        emotion: &str,
        action: Option<String>,
        hardware_required: Option<String>,
        confidence: Option<f32>,
    ) -> Self {
        Self {
            intent: intent.to_string(),
            emotion: emotion.to_string(),
            action,
            hardware_required,
            confidence,
        }
    }
}
