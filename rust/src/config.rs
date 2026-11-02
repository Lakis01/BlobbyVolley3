// Blobby Volley 3 - Persistent User Configuration & Profile System
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub player1_name: String,
    pub player2_name: String,
    pub fullscreen: bool,
    pub sound_muted: bool,
    pub score_to_win: i32,
    pub arena_index: usize,
    pub difficulty_index: usize,
    pub rule_mode_index: usize,
    pub left_theme_index: usize,
    pub right_theme_index: usize,
    #[serde(default)]
    pub matchmaking_registry_url: Option<String>,
}

impl Default for UserConfig {
    fn default() -> Self {
        // Detect default username from environment or fallback to "gratis01"
        let p1_default = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "gratis01".to_string());

        Self {
            player1_name: p1_default,
            player2_name: "Player 2".to_string(),
            fullscreen: false,
            sound_muted: false,
            score_to_win: 15,
            arena_index: 0,
            difficulty_index: 2, // Regular (Medium)
            rule_mode_index: 0,   // Modern Rally
            left_theme_index: 0,
            right_theme_index: 1,
            matchmaking_registry_url: None,
        }
    }
}

impl UserConfig {
    fn config_path() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            let mut p = PathBuf::from(home);
            p.push(".config");
            p.push("blobbyvolley3");
            let _ = fs::create_dir_all(&p);
            p.push("config.json");
            p
        } else {
            PathBuf::from("config.json")
        }
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str::<UserConfig>(&data) {
                return cfg;
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Ok(data) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, data);
        }
    }
}

