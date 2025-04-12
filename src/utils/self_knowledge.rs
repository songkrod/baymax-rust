use std::sync::OnceLock;
use std::env;
use std::fs;
use serde_json::{Value, json};

static RAW_JSON: OnceLock<String> = OnceLock::new();
static PARSED_JSON: OnceLock<Value> = OnceLock::new();
static SUMMARY_TEXT: OnceLock<String> = OnceLock::new();

/// โหลด raw string ของ self_knowledge.json
pub fn load_json_raw() -> &'static str {
    RAW_JSON.get_or_init(|| {
        let path = env::var("self_knowledge_path")
            .unwrap_or_else(|_| "src/data/memory/self_knowledge.json".to_string());
        fs::read_to_string(&path).unwrap_or_else(|_| "{}".to_string())
    })
}

/// โหลดเป็น JSON object
pub fn load_json_object() -> &'static Value {
    PARSED_JSON.get_or_init(|| {
        let raw = load_json_raw();
        serde_json::from_str(raw).unwrap_or(json!({}))
    })
}

/// คืนค่า JSON minified เป็น string (เหมาะกับ log)
pub fn load_json_minified() -> String {
    serde_json::to_string(&load_json_object()).unwrap_or_else(|_| "{}".to_string())
}

/// สร้าง summary เป็นข้อความที่อ่านง่าย ใช้กับ GPT prompt
pub fn load_summary_text() -> &'static str {
    SUMMARY_TEXT.get_or_init(|| {
        let json = load_json_object();
        let mut lines = vec![];

        fn indent_list(label: &str, list: &Vec<Value>) -> Vec<String> {
            let mut out = vec![format!("{}:", label)];
            for item in list {
                if let Some(s) = item.as_str() {
                    out.push(format!("  • {}", s));
                } else if item.is_object() {
                    out.push(format!("  • {}", serde_json::to_string(item).unwrap_or_default()));
                }
            }
            out
        }

        fn add_if_str(map: &Value, key: &str, label: &str, out: &mut Vec<String>) {
            if let Some(s) = map.get(key).and_then(|v| v.as_str()) {
                out.push(format!("{}: {}", label, s));
            }
        }

        // Basic
        add_if_str(json, "name", "Name", &mut lines);
        add_if_str(json, "model", "Model", &mut lines);
        add_if_str(json, "version", "Software Version", &mut lines);

        // Hardware
        if let Some(hw) = json.get("hardware") {
            lines.push("\nHardware:".to_string());

            if let Some(cpu) = hw.get("main_processor").and_then(|v| v.as_str()) {
                lines.push(format!("- CPU: {}", cpu));
            }

            if let Some(audio) = hw.get("audio") {
                if let Some(mic) = audio.get("microphone").and_then(|v| v.get("model")).and_then(|v| v.as_str()) {
                    lines.push(format!("- Microphone: {}", mic));
                }
                if let Some(spk) = audio.get("speaker").and_then(|v| v.get("type")).and_then(|v| v.as_str()) {
                    lines.push(format!("- Speaker: {}", spk));
                }
            }

            if let Some(arr) = hw.get("sensors").and_then(|v| v.as_array()) {
                lines.extend(indent_list("- Sensors", arr));
            }

            if let Some(arr) = hw.get("actuators").and_then(|v| v.as_array()) {
                lines.extend(indent_list("- Actuators", arr));
            }
        }

        // Abilities / Limitations
        if let Some(arr) = json.get("abilities").and_then(|v| v.as_array()) {
            lines.push("\nAbilities:".to_string());
            lines.extend(arr.iter().filter_map(|v| v.as_str()).map(|s| format!("- {}", s)));
        }

        if let Some(arr) = json.get("limitations").and_then(|v| v.as_array()) {
            lines.push("\nLimitations:".to_string());
            lines.extend(arr.iter().filter_map(|v| v.as_str()).map(|s| format!("- {}", s)));
        }

        // Custom fields
        if let Some(note) = json.get("note").and_then(|v| v.as_str()) {
            lines.push(format!("\nNote: {}", note));
        }

        lines.join("\n")
    })
}