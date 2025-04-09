use serde::{Deserialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ReasoningResult {
    pub intent: String,
    pub emotion: String,
    pub reply: String,

    #[serde(default)]
    pub follow_up: Option<String>,

    #[serde(default)]
    pub action: Option<String>,

    #[serde(default)]
    pub hardware_required: Option<String>,

    #[serde(default)]
    pub confidence: Option<f32>,  // Use Option<f32> to handle missing or null values
}

impl ReasoningResult {
    pub fn error(intent: &str, reply: &str) -> Self {
        Self {
            intent: intent.to_string(),
            emotion: "neutral".to_string(),
            reply: reply.to_string(),
            follow_up: None,
            action: None,
            hardware_required: None,
            confidence: None,
        }
    }

    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    pub fn new(intent: &str, emotion: &str, reply: &str, follow_up: Option<String>, action: Option<String>, hardware_required: Option<String>, confidence: Option<f32>) -> Self {
        Self {
            intent: intent.to_string(),
            emotion: emotion.to_string(),
            reply: reply.to_string(),
            follow_up,
            action,
            hardware_required,
            confidence,
        }
    }
}