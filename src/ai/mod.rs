use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const UNCLOSEAI_URL: &str = "https://hermes.ai.unturf.com/v1/chat/completions";
const UNCLOSEAI_MODEL: &str = "turboderp/Qwen3.8-27B-exl3";

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub command_id: String,
    pub reason: String,
    pub score: f32,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: u32,
    temperature: f32,
    stream: bool,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

pub fn ask_ai(query: &str, commands: &[(String, String, String, String)]) -> Result<Vec<Suggestion>> {
    // Pre-filter: keep only commands whose name/command/description loosely match the query keywords
    // Simple approach: score each command by counting how many query words appear in its fields
    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().filter(|w| w.len() > 2).collect();

    let mut scored: Vec<(usize, &(String, String, String, String))> = commands.iter()
        .map(|c| {
            let haystack = format!("{} {} {}", c.1, c.2, c.3).to_lowercase();
            let score = query_words.iter().filter(|w| haystack.contains(*w)).count();
            (score, c)
        })
        .collect();

    // Sort by score descending, then take top 50
    scored.sort_by(|a, b| b.0.cmp(&a.0));
    let top: Vec<_> = scored.into_iter().take(50).collect();

    let mut catalog = String::new();
    for (_, (id, name, cmd, _desc)) in top {
        catalog.push_str(&format!("- {} | {}\n", id, cmd));
    }

    let system_prompt = "You are a command finder API. Return ONLY a JSON array, no other text. Format: [{\"id\":\"...\",\"reason\":\"...\",\"score\":0.9}]. Maximum 3 items.";

    let user_prompt = format!(
        "Query: {}\n\nAvailable commands:\n{}\n\nReturn the top 3 matches as a JSON array:",
        query, catalog
    );

    let request = ChatRequest {
        model: UNCLOSEAI_MODEL.to_string(),
        messages: vec![
            Message { role: "system".to_string(), content: system_prompt.to_string() },
            Message { role: "user".to_string(), content: user_prompt },
        ],
        max_tokens: 500,
        temperature: 0.3,
        stream: false,
    };

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .context("Failed to build HTTP client")?;

    let response: ChatResponse = client
        .post(UNCLOSEAI_URL)
        .json(&request)
        .send()
        .context("Failed to call uncloseai")?
        .json()
        .context("Failed to parse uncloseai response")?;

    let content = response.choices.first()
        .map(|c| c.message.content.clone())
        .unwrap_or_default();

    if content.is_empty() {
        anyhow::bail!(
            "AI returned empty content. Number of choices: {}, finish_reason may be length or content_filter. Full response was not captured.",
            response.choices.len()
        );
    }

    let cleaned = content
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    if cleaned.is_empty() {
        anyhow::bail!(
            "AI returned empty content. Raw response.choices[0].message.content was: {:?}",
            content
        );
    }

    #[derive(Deserialize)]
    struct RawSuggestion {
        id: String,
        reason: String,
        score: f32,
    }

    let raw: Vec<RawSuggestion> = match serde_json::from_str(cleaned) {
        Ok(v) => v,
        Err(e) => {
            anyhow::bail!(
                "Parse error: {} | Content length: {} chars | Raw (last 500 chars): ...{}",
                e,
                cleaned.len(),
                &cleaned[cleaned.len().saturating_sub(500)..]
            );
        }
    };

    Ok(raw.into_iter().map(|r| Suggestion {
        command_id: r.id,
        reason: r.reason,
        score: r.score,
    }).collect())
}
