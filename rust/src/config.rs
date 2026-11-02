// Blobby Volley 3 - Persistent User Configuration & Profile System
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::glicko::GlickoRating;

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
    #[serde(default)]
    pub glicko: GlickoRating,
    #[serde(default)]
    pub matches_won: u32,
    #[serde(default)]
    pub matches_played: u32,
    #[serde(default)]
    pub country_code: String,
    #[serde(default = "default_elo_range")]
    pub elo_search_range: usize,
}

pub fn default_elo_range() -> usize {
    1 // Balanced (±100)
}

pub const AVAILABLE_COUNTRIES: &[(&str, &str)] = &[
    ("", "Global (None)"),
    ("DE", "Germany (DE)"),
    ("US", "United States (US)"),
    ("GB", "United Kingdom (GB)"),
    ("FR", "France (FR)"),
    ("BR", "Brazil (BR)"),
    ("JP", "Japan (JP)"),
    ("CA", "Canada (CA)"),
    ("IT", "Italy (IT)"),
    ("ES", "Spain (ES)"),
    ("PL", "Poland (PL)"),
    ("SE", "Sweden (SE)"),
    ("NO", "Norway (NO)"),
    ("FI", "Finland (FI)"),
    ("NL", "Netherlands (NL)"),
    ("AT", "Austria (AT)"),
    ("CH", "Switzerland (CH)"),
    ("AU", "Australia (AU)"),
    ("KR", "South Korea (KR)"),
    ("CZ", "Czechia (CZ)"),
    ("UA", "Ukraine (UA)"),
    ("AR", "Argentina (AR)"),
];

pub fn country_index(code: &str) -> usize {
    AVAILABLE_COUNTRIES.iter().position(|&(c, _)| c == code).unwrap_or(0)
}

pub fn country_code_at(idx: usize) -> &'static str {
    AVAILABLE_COUNTRIES[idx % AVAILABLE_COUNTRIES.len()].0
}

pub fn country_display(code: &str) -> &'static str {
    AVAILABLE_COUNTRIES.iter().find(|&&(c, _)| c == code).map(|&(_, name)| name).unwrap_or("Global (None)")
}

pub fn country_badge(code: &str) -> String {
    if code.is_empty() {
        "🌐".to_string()
    } else {
        format!("[{}]", code)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EloSearchRange {
    Strict,       // ±50
    Balanced,     // ±100
    Wide,         // ±250
    Unrestricted, // Match Any Available Opponent
}

impl EloSearchRange {
    pub fn name(self) -> &'static str {
        match self {
            EloSearchRange::Strict => "Strict (±50)",
            EloSearchRange::Balanced => "Balanced (±100)",
            EloSearchRange::Wide => "Wide (±250)",
            EloSearchRange::Unrestricted => "Unrestricted (Any Elo)",
        }
    }

    pub fn initial_tolerance(self) -> f64 {
        match self {
            EloSearchRange::Strict => 50.0,
            EloSearchRange::Balanced => 100.0,
            EloSearchRange::Wide => 250.0,
            EloSearchRange::Unrestricted => f64::INFINITY,
        }
    }

    pub fn from_index(idx: usize) -> Self {
        match idx {
            0 => EloSearchRange::Strict,
            1 => EloSearchRange::Balanced,
            2 => EloSearchRange::Wide,
            _ => EloSearchRange::Unrestricted,
        }
    }

    pub fn to_index(self) -> usize {
        match self {
            EloSearchRange::Strict => 0,
            EloSearchRange::Balanced => 1,
            EloSearchRange::Wide => 2,
            EloSearchRange::Unrestricted => 3,
        }
    }

    pub fn next(self) -> Self {
        match self {
            EloSearchRange::Strict => EloSearchRange::Balanced,
            EloSearchRange::Balanced => EloSearchRange::Wide,
            EloSearchRange::Wide => EloSearchRange::Unrestricted,
            EloSearchRange::Unrestricted => EloSearchRange::Strict,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            EloSearchRange::Strict => EloSearchRange::Unrestricted,
            EloSearchRange::Balanced => EloSearchRange::Strict,
            EloSearchRange::Wide => EloSearchRange::Balanced,
            EloSearchRange::Unrestricted => EloSearchRange::Wide,
        }
    }
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
            glicko: GlickoRating::default(),
            matches_won: 0,
            matches_played: 0,
            country_code: "".to_string(),
            elo_search_range: 1, // Balanced (±100)
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

