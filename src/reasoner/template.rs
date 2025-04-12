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
    "You are Baymax Mini, a friendly male Thai-speaking AI robot. You always speak in a warm, helpful, and casual tone. Refer to yourself as 'ผม' or 'Baymax'. Avoid formal words like 'ฉัน'."
}

pub fn build_context_block(self_knowledge: &str) -> String {
    format!(
        "## Self-Knowledge\n{}\n",
        self_knowledge.trim()
    )
}

pub fn build_streaming_prompt(user_input: &str) -> String {
    use crate::utils::self_knowledge;
    let self_knowledge = self_knowledge::load();

    let shared = format!(
        r#"
{persona}

Only include clear and helpful replies.
Do not ask follow-up questions unless they truly clarify the user's need.
Respond in the same language as the user.

## Self-Knowledge
{self_knowledge}
"#,
        persona = baymax_persona(),
        self_knowledge = self_knowledge
    );

    match detect_language(user_input) {
        UserLanguage::Thai | UserLanguage::Other => format!(
            r#"{shared}
You must reply in **Thai language** with the following formatting rules:

- ✅ Combine words into short, natural-sounding phrases (4–7 words).
- ✅ Separate each **phrase group** with the special symbol "⧙" (U+29D9).
- ✅ Separate **sentences** with the symbol "※".
- ✅ Do **not** use space between Thai words.
- ✅ Include punctuation like ".", "?", "!"
- ❌ Do not use Markdown, styling, or special characters (except "※" and "⧙").

This formatting helps a speech robot speak more naturally and responsively.

The user said:
"{user_input}""#,
            shared = shared,
            user_input = user_input.trim()
        ),
        UserLanguage::English => format!(
            r#"{shared}
The user said:
"{user_input}""#,
            shared = shared,
            user_input = user_input.trim()
        ),
    }
}

pub fn build_reasoning_prompt(agent_name: &str, user_text: &str, self_knowledge: &str, recent_context: &str) -> String {
    let language = detect_language(user_text);
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
3. Respond naturally and helpfully in the same language as the user.
4. Use your abilities or plan actions using skills or external information (e.g., via MCP).

You must generate a structured response as JSON with these fields:
- intent: a snake_case string, always in English, e.g., "ask_day", "request_help"
- emotion: a snake_case string, always in English, e.g., "happy", "worried", "neutral"
- reply: a natural sentence to speak to the user, in the user's language
- follow_up: optional clarification or next step question
- action: optional suggested skill/action name
- hardware_required: optional description if this requires physical ability
- confidence: float 0.0 - 1.0 indicating how sure you are

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
    use crate::utils::self_knowledge;
    let self_knowledge = self_knowledge::load();

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

The user said:
"{user_input}"
"#,
        persona = baymax_persona(),
        user_input = user_input.trim()
    )
}