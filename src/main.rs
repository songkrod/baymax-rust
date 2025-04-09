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

use std::sync::Arc;
use tokio::sync::Mutex;

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

    let agent = AiAgent::new(&agent_name);

    loop {
        wait_for_wake_word(&agent, &memory_queue).await;
        wait_for_command(&agent, &memory_queue, &context).await;
    }
}

async fn wait_for_wake_word(agent: &AiAgent, memory_queue: &MemoryQueue) {
    agent.say("สวัสดีครับ สามารถเรียกผมได้เลยครับ").await;

    loop {
        info!("😴 [Sleep Mode] รอคำปลุกที่มีชื่อหุ่น...");

        if let Some(transcript) = agent.listen().await {
            if let Some(name) = is_called_by_name(&transcript) {
                info!("👂 ถูกเรียกชื่อว่า: {}", name);
                memory_queue.enqueue("wake_phrase", &transcript.clone()).await;
                agent.say("สวัสดีครับ ผมตื่นแล้วครับ").await;
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
    loop {
        info!("🟢 [Active Mode] รอฟังคำสั่งจากผู้ใช้...");

        if let Some(transcript) = agent.listen().await {
            memory_queue.enqueue("last_query", &transcript.clone()).await;

            if transcript.contains("นอน") || transcript.contains("พักก่อน") {
                agent.say("งั้นผมขอพักก่อนนะครับ").await;
                break;
            }

            let prompt_context = context.get_context_prompt();
            let result = agent.reason(&transcript, Some(&prompt_context)).await;

            agent.say(&result.reply).await;

            if let Some(follow_up) = &result.follow_up {
                if !follow_up.trim().is_empty() {
                    agent.say(follow_up).await;
                }
            }

            context.append(&transcript, &result.reply);
            context.trim_oldest(20);
            context.save_to_file();

            let intent = result.intent.clone();
            let memory_queue_clone = memory_queue.clone();
            tokio::spawn(async move {
                memory_queue_clone.enqueue("last_intent", &intent).await;
            });
        } else {
            warn!("📭 ไม่ได้ยินอะไรเลย");
        }
    }
}