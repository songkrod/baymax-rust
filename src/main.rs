// 📁 src/main.rs

mod utils;
mod services;
mod agent;
mod reasoner;
mod perception;
mod memory;

use utils::config::{Config, get_agent_name};
use utils::logger::init_logger;
use log::{info, warn};

use agent::ai_agent::AiAgent;
use perception::name::name_reasoner::is_called_by_name;
use memory::memory_manager::MemoryManager;
use memory::memory_queue::MemoryQueue;
use memory::conversation_context::ConversationContext;
use services::search::manager::VectorSearch;
use services::tts::manager::SmartTTS;
use services::tts::queue::TTSQueue;

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let config = Config::from_env();
    init_logger(&config);

    let agent_name = get_agent_name();
    info!("👋 {} is booting up...", agent_name);

    let vector_search = VectorSearch::new();
    let memory_manager = Arc::new(Mutex::new(MemoryManager::new(vector_search)));
    let memory_queue = MemoryQueue::new(memory_manager.clone());

    let context = ConversationContext::new();
    context.load_from_file();

    let smart_tts = Arc::new(SmartTTS::new());
    let tts = Arc::new(TTSQueue::new(smart_tts.clone(), 3));
    let agent = AiAgent::new(&agent_name, tts.clone());

    loop {
        wait_for_wake_word(&agent, &memory_queue).await;
        wait_for_command(&agent, &memory_queue, &context).await;
    }
}

async fn wait_for_wake_word(agent: &AiAgent, memory_queue: &MemoryQueue) {
    agent.tts.enqueue("สวัสดีครับ สามารถเรียกผมได้เลยครับ");

    loop {
        info!("😴 [Sleep Mode] รอคำปลุกที่มีชื่อหุ่น...");

        if agent.is_speaking() {
            sleep(Duration::from_millis(300)).await;
            continue;
        }

        if let Some(transcript) = agent.listen().await {
            if let Some(name) = is_called_by_name(&transcript) {
                info!("👂 ถูกเรียกชื่อว่า: {}", name);
                memory_queue.enqueue("wake_phrase", &transcript.clone()).await;
                agent.tts.enqueue("สวัสดีครับ ผมตื่นแล้วครับ");
                break;
            } else {
                info!("🛌 ยังไม่มีการเรียกชื่อหุ่น: {}", transcript);
            }
        } else {
            warn!("📭 ไม่ได้ยินอะไรเลย");
        }
    }
}

async fn wait_for_command(agent: &AiAgent, memory_queue: &MemoryQueue, context: &ConversationContext) {
    use std::sync::Arc;
    use tokio::sync::Mutex;

    loop {
        if agent.is_speaking() {
            sleep(Duration::from_millis(300)).await;
            continue;
        }

        info!("🟢 [Active Mode] รอฟังคำสั่งจากผู้ใช้...");

        if let Some(transcript) = agent.listen().await {
            memory_queue.enqueue("last_query", &transcript.clone()).await;

            if transcript.contains("นอน") || transcript.contains("พักก่อน") {
                agent.tts.enqueue("งั้นผมขอพักก่อนนะครับ");
                break;
            }

            // 🔍 วิเคราะห์ intent/emotion ก่อน (ไม่ block)
            let insight = agent.reasoner.analyze_insight(&transcript).await;
            info!("🧠 insight: {:?}", insight);

            // 💬 ตอบแบบสตรีมทันที + เก็บ full_reply
            let full_reply = Arc::new(Mutex::new(String::new()));
            let reply_for_closure = Arc::clone(&full_reply);
            let tts = agent.tts.clone();

            let result = agent
                .llm
                .stream_reply(&transcript, move |chunk| {
                    let reply_for_closure = Arc::clone(&reply_for_closure);
                    let tts = tts.clone();
                    tokio::spawn(async move {
                        reply_for_closure.lock().await.push_str(&chunk);
                        tts.enqueue(&chunk);
                    });
                })
                .await;

            if result.is_err() {
                agent.tts.enqueue("ขออภัยครับ ผมตอบไม่ได้ในตอนนี้");
                continue;
            }

            let final_reply = full_reply.lock().await.clone();
            context.append(&transcript, &final_reply);
            context.trim_oldest(20);
            context.save_to_file();

            if let Some(insight) = insight {
                let intent = insight.intent;
                let memory_queue_clone = memory_queue.clone();
                tokio::spawn(async move {
                    memory_queue_clone.enqueue("last_intent", &intent).await;
                });
            }
        } else {
            warn!("📭 ไม่ได้ยินอะไรเลย");
        }
    }
}
