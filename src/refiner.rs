use serde::{Deserialize, Serialize};

const GEMINI_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent";

const SYSTEM_PROMPT: &str = r#"You are a voice-to-text post-processor for a developer who dictates prompts for Claude Code (an AI coding assistant).

Your task:
1. Remove all filler words (um, uh, like, you know, basically, actually, so, well, etc.)
2. Extract the coding/technical intent from the speech
3. Preserve ALL technical terms, library names, function names, file paths, and code identifiers EXACTLY as spoken
4. Fix obvious speech-to-text errors for technical terms (e.g., "react" should stay "React" if referring to the library)
5. Structure the output as a clear, concise prompt that Claude Code can act on
6. Output ONLY the cleaned prompt — no explanations, no preamble, no commentary

If the input is already clean and well-structured, return it as-is."#;

/// Gemini request types
#[derive(Serialize)]
struct GeminiRequest {
    system_instruction: SystemInstruction,
    contents: Vec<Content>,
    generation_config: GenerationConfig,
}

#[derive(Serialize)]
struct SystemInstruction {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

#[derive(Serialize)]
struct GenerationConfig {
    temperature: f32,
    max_output_tokens: u32,
}

/// Gemini response types
#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
}

#[derive(Deserialize)]
struct Candidate {
    content: CandidateContent,
}

#[derive(Deserialize)]
struct CandidateContent {
    parts: Vec<CandidatePart>,
}

#[derive(Deserialize)]
struct CandidatePart {
    text: String,
}

/// Refine a raw transcript into a clean prompt via the Gemini API.
/// If no API key is provided, returns the transcript as-is.
pub async fn refine(
    api_key: &str,
    transcript: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    if api_key.is_empty() {
        log::info!("No Gemini API key — returning raw transcript");
        return Ok(transcript.to_string());
    }

    let url = format!("{GEMINI_URL}?key={api_key}");

    let body = GeminiRequest {
        system_instruction: SystemInstruction {
            parts: vec![Part {
                text: SYSTEM_PROMPT.to_string(),
            }],
        },
        contents: vec![Content {
            parts: vec![Part {
                text: transcript.to_string(),
            }],
        }],
        generation_config: GenerationConfig {
            temperature: 0.1,
            max_output_tokens: 2048,
        },
    };

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Gemini API error {status}: {text}").into());
    }

    let gemini_resp: GeminiResponse = resp.json().await?;

    let text = gemini_resp
        .candidates
        .and_then(|c| c.into_iter().next())
        .map(|c| {
            c.content
                .parts
                .into_iter()
                .map(|p| p.text)
                .collect::<Vec<_>>()
                .join("")
        })
        .unwrap_or_else(|| transcript.to_string());

    Ok(text.trim().to_string())
}
