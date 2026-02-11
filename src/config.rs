use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Key codes for the hotkey combination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    /// evdev key codes for modifier keys (e.g. 29 = KEY_LEFTCTRL)
    pub modifiers: Vec<u16>,
    /// evdev key code for the trigger key (e.g. 57 = KEY_SPACE)
    pub trigger: u16,
    /// Human-readable name like "Ctrl+Space"
    pub display_name: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            modifiers: vec![29], // KEY_LEFTCTRL
            trigger: 57,         // KEY_SPACE
            display_name: "Ctrl+Space".into(),
        }
    }
}

/// Top-level application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub hotkey: HotkeyConfig,
    pub gemini_api_key: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hotkey: HotkeyConfig::default(),
            gemini_api_key: String::new(),
        }
    }
}

impl Config {
    /// Directory: ~/.config/voice-prompt/
    fn dir() -> PathBuf {
        let mut p = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        p.push("voice-prompt");
        p
    }

    fn path() -> PathBuf {
        Self::dir().join("config.json")
    }

    /// Load from disk, returning defaults if file doesn't exist or is invalid.
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
}
