use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// A single recorded prompt with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptRecord {
    pub text: String,
    pub word_count: usize,
    pub timestamp: String,
}

/// Persistent usage statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Stats {
    pub total_words: usize,
    pub total_prompts: usize,
    #[serde(default)]
    pub history: Vec<PromptRecord>,
}

impl Stats {
    /// Directory: ~/.local/share/voice-prompt/
    fn dir() -> PathBuf {
        let mut p = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
        p.push("voice-prompt");
        p
    }

    fn path() -> PathBuf {
        Self::dir().join("stats.json")
    }

    /// Load from disk, returning defaults if missing.
    pub fn load() -> Self {
        let path = Self::path();
        match fs::read_to_string(&path) {
            Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Persist to disk.
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let dir = Self::dir();
        fs::create_dir_all(&dir)?;
        let data = serde_json::to_string_pretty(self)?;
        fs::write(Self::path(), data)?;
        Ok(())
    }

    /// Record a completed prompt and its word count.
    pub fn record_prompt(&mut self, text: &str) {
        let word_count = text.split_whitespace().count();
        self.total_prompts += 1;
        self.total_words += word_count;
        self.history.push(PromptRecord {
            text: text.to_string(),
            word_count,
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        });
    }
}
