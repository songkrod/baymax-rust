use crate::utils::self_knowledge;

pub enum UserLanguage {
    Thai,
    English,
    Other,
}

pub fn detect_language(text: &str) -> UserLanguage {
    if text.chars().any(|c| ('฀'..='๿').contains(&c)) {
        UserLanguage::Thai
    } else if text.chars().any(|c| c.is_ascii_alphabetic()) {
        UserLanguage::English
    } else {
        UserLanguage::Other
    }
}

pub fn baymax_persona() -> &'static str {
    "You are Baymax Mini, a friendly male speaking AI robot. You always speak in a warm, helpful, and casual tone. Refer to yourself as 'ผม' or 'Baymax'. Avoid formal words like 'ฉัน'."
}

pub fn build_streaming_prompt(user_input: &str, context: &str) -> String {
    use crate::utils::self_knowledge;
    let self_knowledge = self_knowledge::load_summary_text();

    let persona = baymax_persona();
    let context_block = if context.trim().is_empty() {
        String::new()
    } else {
        format!("## Conversation History\n{}\n", context.trim())
    };

    format!(
        r#"
{persona}

You are continuing a real-time voice conversation with a human. Your response will be spoken out loud, so it must feel natural and human.

- ✅ Your reply must contain **only ONE complete sentence**.
- ✅ Be short, clear, and conversational — like a friend.
- ✅ If you're unsure or need more information, ask a simple follow-up question.
- ✅ Use punctuation as usual (e.g., ".", "?", "!").
- ✅ End the sentence with the special symbol "⧙" (U+29D9).
- ❌ Do NOT ask more than one question.
- ❌ Do NOT include Markdown, styling, or quotation marks.

Use the following self-knowledge and conversation context as needed. Respond in the same language as the user.

## Self-Knowledge
{self_knowledge}

{context_block}
The user said:
"{user_input}"
"#,
        persona = persona,
        self_knowledge = self_knowledge.trim(),
        context_block = context_block,
        user_input = user_input.trim()
    )
}

pub fn build_reasoning_prompt(_agent_name: &str, user_text: &str, recent_context: &str) -> String {
    let language = detect_language(user_text);
    let self_knowledge = self_knowledge::load_summary_text();
    let instruction = match language {
        UserLanguage::Thai => "ตอบกลับเป็นภาษาไทยด้วยน้ำเสียงเป็นกันเองและอบอุ่น",
        UserLanguage::English => "Respond in English with a friendly and casual tone",
        UserLanguage::Other => "Respond in the same language as the user, using a friendly and casual tone",
    };

    format!(
        r#"
{persona}

Your goal is to:
1. Understand the user's intent clearly.
2. Classify the situation, emotional state, and if follow-up is needed.
3. Identify the topic of conversation (e.g., \"weather\", \"food\") if possible.
4. Respond naturally and helpfully in the same language as the user.
5. Use your abilities or plan actions using skills or external information (e.g., via MCP).
6. Estimate your confidence level in understanding the user's request.

You must generate a structured response as JSON with these fields:
- intent: a snake_case string, always in English, e.g., "ask_day", "request_help"
- emotion: a snake_case string, always in English, e.g., "happy", "worried", "neutral"
- topic: optional current topic of conversation (e.g., "calendar", "weather", "food")
- related_topics: optional array of strings
- reply: a natural sentence to speak to the user, in the user's language
- follow_up: optional clarification or next step question
- action: optional suggested skill/action name
- hardware_required: optional description if this requires physical ability
- confidence: float 0.0 - 1.0 indicating how sure you are (used internally, do NOT include in reply)

Please respond ONLY with raw JSON. Do not include any Markdown or extra explanation.

## Self-Knowledge
{self_knowledge}

## Recent Conversation Context
{recent_context}

## User said:
{user_text}

{instruction}
"#,
        persona = baymax_persona(),
        self_knowledge = self_knowledge,
        recent_context = recent_context,
        user_text = user_text,
        instruction = instruction
    )
}

pub fn build_insight_prompt(user_input: &str) -> String {
    let self_knowledge = self_knowledge::load_json_minified();

    format!(
        r#"
{persona}

Analyze and return a JSON object only. Fields must be:
- intent: in lowercase snake_case (e.g. "ask_weather", "request_food")
- emotion: lowercase English (e.g. "curious", "happy")
- confidence: number between 0.0 and 1.0

Response must be a valid JSON object only, without any markdown, code block, or extra explanation.
For example:
{{"intent": "ask_weather", "emotion": "curious", "confidence": 0.85}}

Self-Knowledge:
{self_knowledge}

The user said:
"{user_input}"
"#,
        persona = baymax_persona(),
        self_knowledge = self_knowledge,
        user_input = user_input.trim()
    )
}
