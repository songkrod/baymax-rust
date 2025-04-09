use std::env;
use std::fs;

pub fn load() -> String {
    let path = env::var("self_knowledge_path")
        .unwrap_or_else(|_| "src/data/memory/self_knowledge.json".to_string());
    fs::read_to_string(&path).unwrap_or_else(|_| "{}".to_string())
}