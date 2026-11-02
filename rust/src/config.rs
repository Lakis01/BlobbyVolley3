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
    #[serde(default)]
    pub high_ball_camera: usize, // 0 = Indicator, 1 = DynamicZoom
    #[serde(default)]
    pub court_size: usize,       // Legacy discrete index
    #[serde(default)]
    pub court_width_factor: f32, // 0.0 = Standard (800px), 1.0 = Max (Window/Screen borders)
}

impl UserConfig {
    pub fn get_court_width(&self, aspect: f32) -> f32 {
        let max_w = (600.0 * aspect).max(800.0);
        let factor = self.court_width_factor.clamp(0.0, 1.0);
        (800.0 + (max_w - 800.0) * factor).round()
    }
}

pub fn default_elo_range() -> usize {
    1 // Balanced (±100)
}

pub const AVAILABLE_COUNTRIES: &[(&str, &str)] = &[
    ("", "Global (None)"),
    ("AF", "Afghanistan (AF)"),
    ("AL", "Albania (AL)"),
    ("DZ", "Algeria (DZ)"),
    ("AS", "American Samoa (AS)"),
    ("AD", "Andorra (AD)"),
    ("AO", "Angola (AO)"),
    ("AI", "Anguilla (AI)"),
    ("AQ", "Antarctica (AQ)"),
    ("AG", "Antigua & Barbuda (AG)"),
    ("AR", "Argentina (AR)"),
    ("AM", "Armenia (AM)"),
    ("AW", "Aruba (AW)"),
    ("AU", "Australia (AU)"),
    ("AT", "Austria (AT)"),
    ("AZ", "Azerbaijan (AZ)"),
    ("BS", "Bahamas (BS)"),
    ("BH", "Bahrain (BH)"),
    ("BD", "Bangladesh (BD)"),
    ("BB", "Barbados (BB)"),
    ("BY", "Belarus (BY)"),
    ("BE", "Belgium (BE)"),
    ("BZ", "Belize (BZ)"),
    ("BJ", "Benin (BJ)"),
    ("BM", "Bermuda (BM)"),
    ("BT", "Bhutan (BT)"),
    ("BO", "Bolivia (BO)"),
    ("BA", "Bosnia (BA)"),
    ("BW", "Botswana (BW)"),
    ("BV", "Bouvet Island (BV)"),
    ("BR", "Brazil (BR)"),
    ("IO", "British Indian Ocean Territory (IO)"),
    ("VG", "British Virgin Islands (VG)"),
    ("BN", "Brunei (BN)"),
    ("BG", "Bulgaria (BG)"),
    ("BF", "Burkina Faso (BF)"),
    ("BI", "Burundi (BI)"),
    ("KH", "Cambodia (KH)"),
    ("CM", "Cameroon (CM)"),
    ("CA", "Canada (CA)"),
    ("CV", "Cape Verde (CV)"),
    ("BQ", "Caribbean Netherlands (BQ)"),
    ("KY", "Cayman Islands (KY)"),
    ("CF", "Central African Republic (CF)"),
    ("TD", "Chad (TD)"),
    ("CL", "Chile (CL)"),
    ("CN", "China (CN)"),
    ("CX", "Christmas Island (CX)"),
    ("CC", "Cocos Islands (CC)"),
    ("CO", "Colombia (CO)"),
    ("KM", "Comoros (KM)"),
    ("CG", "Congo - Brazzaville (CG)"),
    ("CD", "Congo - Kinshasa (CD)"),
    ("CK", "Cook Islands (CK)"),
    ("CR", "Costa Rica (CR)"),
    ("HR", "Croatia (HR)"),
    ("CU", "Cuba (CU)"),
    ("CW", "Curaçao (CW)"),
    ("CY", "Cyprus (CY)"),
    ("CZ", "Czechia (CZ)"),
    ("CI", "Côte d’Ivoire (CI)"),
    ("DK", "Denmark (DK)"),
    ("DJ", "Djibouti (DJ)"),
    ("DM", "Dominica (DM)"),
    ("DO", "Dominican Republic (DO)"),
    ("EC", "Ecuador (EC)"),
    ("EG", "Egypt (EG)"),
    ("SV", "El Salvador (SV)"),
    ("GQ", "Equatorial Guinea (GQ)"),
    ("ER", "Eritrea (ER)"),
    ("EE", "Estonia (EE)"),
    ("SZ", "Eswatini (SZ)"),
    ("ET", "Ethiopia (ET)"),
    ("FK", "Falkland Islands (FK)"),
    ("FO", "Faroe Islands (FO)"),
    ("FJ", "Fiji (FJ)"),
    ("FI", "Finland (FI)"),
    ("FR", "France (FR)"),
    ("GF", "French Guiana (GF)"),
    ("PF", "French Polynesia (PF)"),
    ("TF", "French Southern Territories (TF)"),
    ("GA", "Gabon (GA)"),
    ("GM", "Gambia (GM)"),
    ("GE", "Georgia (GE)"),
    ("DE", "Germany (DE)"),
    ("GH", "Ghana (GH)"),
    ("GI", "Gibraltar (GI)"),
    ("GR", "Greece (GR)"),
    ("GL", "Greenland (GL)"),
    ("GD", "Grenada (GD)"),
    ("GP", "Guadeloupe (GP)"),
    ("GU", "Guam (GU)"),
    ("GT", "Guatemala (GT)"),
    ("GG", "Guernsey (GG)"),
    ("GN", "Guinea (GN)"),
    ("GW", "Guinea-Bissau (GW)"),
    ("GY", "Guyana (GY)"),
    ("HT", "Haiti (HT)"),
    ("HM", "Heard & McDonald Islands (HM)"),
    ("HN", "Honduras (HN)"),
    ("HK", "Hong Kong (HK)"),
    ("HU", "Hungary (HU)"),
    ("IS", "Iceland (IS)"),
    ("IN", "India (IN)"),
    ("ID", "Indonesia (ID)"),
    ("IR", "Iran (IR)"),
    ("IQ", "Iraq (IQ)"),
    ("IE", "Ireland (IE)"),
    ("IM", "Isle of Man (IM)"),
    ("IL", "Israel (IL)"),
    ("IT", "Italy (IT)"),
    ("JM", "Jamaica (JM)"),
    ("JP", "Japan (JP)"),
    ("JE", "Jersey (JE)"),
    ("JO", "Jordan (JO)"),
    ("KZ", "Kazakhstan (KZ)"),
    ("KE", "Kenya (KE)"),
    ("KI", "Kiribati (KI)"),
    ("KW", "Kuwait (KW)"),
    ("KG", "Kyrgyzstan (KG)"),
    ("LA", "Laos (LA)"),
    ("LV", "Latvia (LV)"),
    ("LB", "Lebanon (LB)"),
    ("LS", "Lesotho (LS)"),
    ("LR", "Liberia (LR)"),
    ("LY", "Libya (LY)"),
    ("LI", "Liechtenstein (LI)"),
    ("LT", "Lithuania (LT)"),
    ("LU", "Luxembourg (LU)"),
    ("MO", "Macao (MO)"),
    ("MG", "Madagascar (MG)"),
    ("MW", "Malawi (MW)"),
    ("MY", "Malaysia (MY)"),
    ("MV", "Maldives (MV)"),
    ("ML", "Mali (ML)"),
    ("MT", "Malta (MT)"),
    ("MH", "Marshall Islands (MH)"),
    ("MQ", "Martinique (MQ)"),
    ("MR", "Mauritania (MR)"),
    ("MU", "Mauritius (MU)"),
    ("YT", "Mayotte (YT)"),
    ("MX", "Mexico (MX)"),
    ("FM", "Micronesia (FM)"),
    ("MD", "Moldova (MD)"),
    ("MC", "Monaco (MC)"),
    ("MN", "Mongolia (MN)"),
    ("ME", "Montenegro (ME)"),
    ("MS", "Montserrat (MS)"),
    ("MA", "Morocco (MA)"),
    ("MZ", "Mozambique (MZ)"),
    ("MM", "Myanmar (MM)"),
    ("NA", "Namibia (NA)"),
    ("NR", "Nauru (NR)"),
    ("NP", "Nepal (NP)"),
    ("NL", "Netherlands (NL)"),
    ("NC", "New Caledonia (NC)"),
    ("NZ", "New Zealand (NZ)"),
    ("NI", "Nicaragua (NI)"),
    ("NE", "Niger (NE)"),
    ("NG", "Nigeria (NG)"),
    ("NU", "Niue (NU)"),
    ("NF", "Norfolk Island (NF)"),
    ("KP", "North Korea (KP)"),
    ("MK", "North Macedonia (MK)"),
    ("MP", "Northern Mariana Islands (MP)"),
    ("NO", "Norway (NO)"),
    ("OM", "Oman (OM)"),
    ("PK", "Pakistan (PK)"),
    ("PW", "Palau (PW)"),
    ("PS", "Palestine (PS)"),
    ("PA", "Panama (PA)"),
    ("PG", "Papua New Guinea (PG)"),
    ("PY", "Paraguay (PY)"),
    ("PE", "Peru (PE)"),
    ("PH", "Philippines (PH)"),
    ("PN", "Pitcairn (PN)"),
    ("PL", "Poland (PL)"),
    ("PT", "Portugal (PT)"),
    ("PR", "Puerto Rico (PR)"),
    ("QA", "Qatar (QA)"),
    ("RO", "Romania (RO)"),
    ("RU", "Russia (RU)"),
    ("RW", "Rwanda (RW)"),
    ("RE", "Réunion (RE)"),
    ("WS", "Samoa (WS)"),
    ("SM", "San Marino (SM)"),
    ("SA", "Saudi Arabia (SA)"),
    ("SN", "Senegal (SN)"),
    ("RS", "Serbia (RS)"),
    ("SC", "Seychelles (SC)"),
    ("SL", "Sierra Leone (SL)"),
    ("SG", "Singapore (SG)"),
    ("SX", "Sint Maarten (SX)"),
    ("SK", "Slovakia (SK)"),
    ("SI", "Slovenia (SI)"),
    ("SB", "Solomon Islands (SB)"),
    ("SO", "Somalia (SO)"),
    ("ZA", "South Africa (ZA)"),
    ("GS", "South Georgia & South Sandwich Islands (GS)"),
    ("KR", "South Korea (KR)"),
    ("SS", "South Sudan (SS)"),
    ("ES", "Spain (ES)"),
    ("LK", "Sri Lanka (LK)"),
    ("BL", "St. Barthélemy (BL)"),
    ("SH", "St. Helena (SH)"),
    ("KN", "St. Kitts & Nevis (KN)"),
    ("LC", "St. Lucia (LC)"),
    ("MF", "St. Martin (MF)"),
    ("PM", "St. Pierre & Miquelon (PM)"),
    ("VC", "St. Vincent & Grenadines (VC)"),
    ("SD", "Sudan (SD)"),
    ("SR", "Suriname (SR)"),
    ("SJ", "Svalbard & Jan Mayen (SJ)"),
    ("SE", "Sweden (SE)"),
    ("CH", "Switzerland (CH)"),
    ("SY", "Syria (SY)"),
    ("ST", "São Tomé & Príncipe (ST)"),
    ("TW", "Taiwan (TW)"),
    ("TJ", "Tajikistan (TJ)"),
    ("TZ", "Tanzania (TZ)"),
    ("TH", "Thailand (TH)"),
    ("TL", "Timor-Leste (TL)"),
    ("TG", "Togo (TG)"),
    ("TK", "Tokelau (TK)"),
    ("TO", "Tonga (TO)"),
    ("TT", "Trinidad & Tobago (TT)"),
    ("TN", "Tunisia (TN)"),
    ("TM", "Turkmenistan (TM)"),
    ("TC", "Turks & Caicos Islands (TC)"),
    ("TV", "Tuvalu (TV)"),
    ("TR", "Türkiye (TR)"),
    ("UM", "U.S. Outlying Islands (UM)"),
    ("VI", "U.S. Virgin Islands (VI)"),
    ("GB", "UK (GB)"),
    ("US", "US (US)"),
    ("UG", "Uganda (UG)"),
    ("UA", "Ukraine (UA)"),
    ("AE", "United Arab Emirates (AE)"),
    ("UY", "Uruguay (UY)"),
    ("UZ", "Uzbekistan (UZ)"),
    ("VU", "Vanuatu (VU)"),
    ("VA", "Vatican City (VA)"),
    ("VE", "Venezuela (VE)"),
    ("VN", "Vietnam (VN)"),
    ("WF", "Wallis & Futuna (WF)"),
    ("EH", "Western Sahara (EH)"),
    ("YE", "Yemen (YE)"),
    ("ZM", "Zambia (ZM)"),
    ("ZW", "Zimbabwe (ZW)"),
    ("AX", "Åland Islands (AX)"),
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
        let mut chars = code.chars();
        if let (Some(c1), Some(c2)) = (chars.next(), chars.next()) {
            if c1.is_ascii_alphabetic() && c2.is_ascii_alphabetic() {
                let u1 = 0x1F1E6 + (c1.to_ascii_uppercase() as u32 - 'A' as u32);
                let u2 = 0x1F1E6 + (c2.to_ascii_uppercase() as u32 - 'A' as u32);
                if let (Some(ch1), Some(ch2)) = (char::from_u32(u1), char::from_u32(u2)) {
                    return format!("{}{} [{}]", ch1, ch2, code);
                }
            }
        }
        format!("[{}]", code)
    }
}

pub fn country_index_by_prefix(prefix: char) -> Option<usize> {
    let p = prefix.to_ascii_uppercase();
    AVAILABLE_COUNTRIES.iter().position(|&(code, name)| {
        code.starts_with(p) || name.to_ascii_uppercase().starts_with(p)
    })
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
            high_ball_camera: 0, // Off-Screen Indicator by default
            court_size: 0,       // Standard (800px) by default
            court_width_factor: 0.0, // Standard (800px) by default
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CourtSize {
    Standard,  // 800.0 px
    Wide,      // 960.0 px (+20%)
    ExtraWide, // 1120.0 px (+40%)
    Colossal,  // 1280.0 px (+60%)
}

#[allow(dead_code)]
impl CourtSize {
    pub fn width(self) -> f32 {
        match self {
            CourtSize::Standard => 800.0,
            CourtSize::Wide => 960.0,
            CourtSize::ExtraWide => 1120.0,
            CourtSize::Colossal => 1280.0,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            CourtSize::Standard => "Standard (800px)",
            CourtSize::Wide => "Wide (960px)",
            CourtSize::ExtraWide => "Extra Wide (1120px)",
            CourtSize::Colossal => "Colossal (1280px)",
        }
    }

    pub fn from_index(idx: usize) -> Self {
        match idx {
            1 => CourtSize::Wide,
            2 => CourtSize::ExtraWide,
            3 => CourtSize::Colossal,
            _ => CourtSize::Standard,
        }
    }

    pub fn to_index(self) -> usize {
        match self {
            CourtSize::Standard => 0,
            CourtSize::Wide => 1,
            CourtSize::ExtraWide => 2,
            CourtSize::Colossal => 3,
        }
    }

    pub fn next(self) -> Self {
        match self {
            CourtSize::Standard => CourtSize::Wide,
            CourtSize::Wide => CourtSize::ExtraWide,
            CourtSize::ExtraWide => CourtSize::Colossal,
            CourtSize::Colossal => CourtSize::Standard,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            CourtSize::Standard => CourtSize::Colossal,
            CourtSize::Wide => CourtSize::Standard,
            CourtSize::ExtraWide => CourtSize::Wide,
            CourtSize::Colossal => CourtSize::ExtraWide,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighBallCamera {
    Indicator,   // Rock-solid fixed camera + top-edge HUD indicator tracking X position
    DynamicZoom, // Smooth dynamic zoom-out tracking ball in high arc
}

impl HighBallCamera {
    pub fn name(self) -> &'static str {
        match self {
            HighBallCamera::Indicator => "Off-Screen Indicator",
            HighBallCamera::DynamicZoom => "Dynamic Zoom",
        }
    }

    pub fn from_index(idx: usize) -> Self {
        match idx {
            1 => HighBallCamera::DynamicZoom,
            _ => HighBallCamera::Indicator,
        }
    }

    pub fn to_index(self) -> usize {
        match self {
            HighBallCamera::Indicator => 0,
            HighBallCamera::DynamicZoom => 1,
        }
    }

    pub fn next(self) -> Self {
        match self {
            HighBallCamera::Indicator => HighBallCamera::DynamicZoom,
            HighBallCamera::DynamicZoom => HighBallCamera::Indicator,
        }
    }

    pub fn prev(self) -> Self {
        self.next()
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

