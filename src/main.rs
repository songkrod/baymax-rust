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
use memory::conversation_context::ConversationContext;
use services::search::manager::VectorSearch;

#[tokio::main]
async fn main() {
    let config = Config::from_env();
    init_logger(&config);

    let agent_name = get_agent_name();
    info!("\u{1F44B} {} is booting up...", agent_name);

    let vector_search = VectorSearch::new();
    let mut memory_manager = MemoryManager::new(vector_search);
    let context = ConversationContext::new();
    context.load_from_file();

    let agent = AiAgent::new(&agent_name);

    loop {
        wait_for_wake_word(&agent, &mut memory_manager).await;
        wait_for_command(&agent, &mut memory_manager, &context).await;
    }
}

async fn wait_for_wake_word(agent: &AiAgent, memory_manager: &mut MemoryManager) {
    agent.say("สวัสดีครับ สามารถเรียกผมได้เลยครับ").await;

    loop {
        info!("\u{1F634} [Sleep Mode] รอคำปลุกที่มีชื่อหุ่น...");

        if let Some(transcript) = agent.listen().await {
            if let Some(name) = is_called_by_name(&transcript) {
                info!("\u{1F442} ถูกเรียกชื่อว่า: {}", name);
                memory_manager.add_memory("wake_phrase", &transcript).await;
                agent.say("สวัสดีครับ ผมตื่นแล้วครับ").await;
                break;
            } else {
                info!("\u{1F6CC} ยังไม่มีการเรียกชื่อหุ่น: {}", transcript);
            }
        } else {
            warn!("\u{1F4ED} ไม่ได้ยินอะไรเลย");
        }
    }
}

async fn wait_for_command(agent: &AiAgent, memory_manager: &mut MemoryManager, context: &ConversationContext) {
    loop {
        info!("\u{1F7E2} [Active Mode] รอฟังคำสั่งจากผู้ใช้...");

        if let Some(transcript) = agent.listen().await {
            memory_manager.add_memory("last_query", &transcript).await;

            if transcript.contains("นอน") || transcript.contains("พักก่อน") {
                agent.say("งั้นผมขอพักก่อนนะครับ").await;
                break;
            }

            let prompt_context = context.get_context_prompt();
            let result = agent.reason(&transcript, Some(&prompt_context)).await;

            // พูดตอบหลัก
            agent.say(&result.reply).await;

            // ถ้ามี follow-up → พูดต่อ
            if let Some(follow_up) = &result.follow_up {
                if !follow_up.trim().is_empty() {
                    agent.say(follow_up).await;
                }
            }

            // อัปเดตบริบทและเซฟไว้
            context.append(&transcript, &result.reply);
            context.trim_oldest(20);
            context.save_to_file();

            memory_manager.save_memory("last_intent", &result.intent);
        } else {
            warn!("\u{1F4ED} ไม่ได้ยินอะไรเลย");
        }
    }
}