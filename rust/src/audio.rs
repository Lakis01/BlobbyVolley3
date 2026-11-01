// Blobby Volley 3 - Audio Engine with Embedded High-Fidelity & Classic Sound Effects
use macroquad::audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};

pub struct AudioManager {
    pub sound_hit: Option<Sound>,
    pub sound_whistle: Option<Sound>,
    pub sound_score: Option<Sound>,
    pub is_muted: bool,
    pub volume: f32,
}

impl AudioManager {
    pub async fn new() -> Self {
        // Embed classic authentic Blobby Volley audio bytes
        let hit_bytes = include_bytes!("../../data/sounds/bums.wav");
        let whistle_bytes = include_bytes!("../../data/sounds/pfiff.wav");
        let score_bytes = include_bytes!("../../data/sounds/chat.wav");

        let sound_hit = load_sound_from_bytes(hit_bytes).await.ok();
        let sound_whistle = load_sound_from_bytes(whistle_bytes).await.ok();
        let sound_score = load_sound_from_bytes(score_bytes).await.ok();

        Self {
            sound_hit,
            sound_whistle,
            sound_score,
            is_muted: false,
            volume: 0.85,
        }
    }

    pub fn play_ball_hit(&self, intensity: f32) {
        if self.is_muted {
            return;
        }
        if let Some(ref sound) = self.sound_hit {
            let vol = (0.5 + intensity * 0.5) * self.volume;
            play_sound(
                sound,
                PlaySoundParams {
                    looped: false,
                    volume: vol,
                },
            );
        }
    }

    pub fn play_whistle(&self) {
        if self.is_muted {
            return;
        }
        if let Some(ref sound) = self.sound_whistle {
            play_sound(
                sound,
                PlaySoundParams {
                    looped: false,
                    volume: self.volume * 0.9,
                },
            );
        }
    }

    pub fn play_score(&self) {
        if self.is_muted {
            return;
        }
        if let Some(ref sound) = self.sound_score {
            play_sound(
                sound,
                PlaySoundParams {
                    looped: false,
                    volume: self.volume,
                },
            );
        }
    }

    pub fn toggle_mute(&mut self) {
        self.is_muted = !self.is_muted;
    }
}

