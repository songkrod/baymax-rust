// 📁 src/main.rs

mod utils;
mod services;
mod agent;
mod perception;
mod memory;
mod reasoner;
mod hardware;

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
use hardware::speaker::controller::create_speaker_controller_from_env;
use hardware::speaker::interface::SpeakerBackend;
use reasoner::template::build_insight_prompt;

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

    let smart_tts = Arc::new(SmartTTS::new());
    let speaker: Arc<dyn SpeakerBackend> = create_speaker_controller_from_env();

    let (tts_queue, done_rx) = TTSQueue::new(smart_tts.clone(), speaker, 3);
    let tts = Arc::new(tts_queue);

    let agent = AiAgent::new(&agent_name, tts.clone(), done_rx);

    loop {
        wait_for_wake_word(&agent, &memory_queue).await;
        wait_for_command(&agent, &memory_queue, &context, &memory_manager).await;
    }
}

async fn wait_for_wake_word(agent: &AiAgent, memory_queue: &MemoryQueue) {
    agent.say("สวัสดีครับ สามารถเรียกผมได้เลยครับ").await;

    loop {
        info!("😴 [Sleep Mode] รอคำปลุกที่มีชื่อหุ่น...");

        if let Some(transcript) = agent.listen().await {
            if let Some(name) = is_called_by_name(&transcript) {
                info!("🗢 ถูกเรียกชื่อว่า: {}", name);
                memory_queue.enqueue("wake_phrase", &transcript.clone()).await;
                agent.say("สวัสดีครับ ผมตื่นแล้วครับ").await;
                break;
            } else {
                info!("🛌 ยังไม่มีการเรียกชื่อหุ่น: {}", transcript);
            }
        } else {
            warn!("👭 ไม่ได้ยินอะไรเลย");
        }
    }
}

async fn wait_for_command(
    agent: &AiAgent,
    memory_queue: &MemoryQueue,
    context: &ConversationContext,
    memory_manager: &Arc<Mutex<MemoryManager>>,
) {
    loop {
        info!("🟢 [Active Mode] รอฟังคำสั่งจากผู้ใช้...");

        if let Some(transcript) = agent.listen().await {
            memory_queue.enqueue("last_query", &transcript.clone()).await;

            if transcript.contains("นอน") || transcript.contains("พักก่อน") {
                agent.say("งั้นผมขอพักก่อนนะครับ").await;
                break;
            }

            let insight_prompt = build_insight_prompt(&transcript);
            let insight = agent.reasoner.analyze_insight(&insight_prompt).await;
            info!("🧠 insight: {:?}", insight);

            // ✅ เพิ่ม vector context ก่อนพูดจริง
            let vector_matches = memory_manager.lock().await.search_memory(&transcript).await;
            let vector_context = vector_matches.join("\n");
            let recent_context = context.get_context_prompt();
            let full_context = format!("{}\n{}", vector_context, recent_context);

            let final_reply = agent.think_and_say_streaming(&transcript, &full_context).await;
            info!("💬 ตอบคำถาม: {}", final_reply);
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
            warn!("👭 ไม่ได้ยินอะไรเลย");
        }
    }
}
