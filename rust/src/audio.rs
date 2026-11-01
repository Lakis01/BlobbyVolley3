// Blobby Volley 3 - Audio Engine with Embedded High-Fidelity & Classic Sound Effects (Opus Encoded)
use std::io::Cursor;
use macroquad::audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};
use opus_pure::{OggOpusReader, MAX_PACKET_SAMPLES, Trim};

pub struct AudioManager {
    pub sound_hit: Option<Sound>,
    pub sound_whistle: Option<Sound>,
    pub sound_score: Option<Sound>,
    pub is_muted: bool,
    pub volume: f32,
}

impl AudioManager {
    pub async fn new() -> Self {
        // Embed Opus-encoded audio assets (bitrate 100 kbps)
        let hit_opus = include_bytes!("../assets/sounds/bums.opus");
        let whistle_opus = include_bytes!("../assets/sounds/pfiff.opus");
        let score_opus = include_bytes!("../assets/sounds/chat.opus");

        let sound_hit = if let Ok(wav) = decode_opus_to_wav(hit_opus) {
            load_sound_from_bytes(&wav).await.ok()
        } else {
            None
        };

        let sound_whistle = if let Ok(wav) = decode_opus_to_wav(whistle_opus) {
            load_sound_from_bytes(&wav).await.ok()
        } else {
            None
        };

        let sound_score = if let Ok(wav) = decode_opus_to_wav(score_opus) {
            load_sound_from_bytes(&wav).await.ok()
        } else {
            None
        };

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
                    volume: self.volume,
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
                    volume: self.volume * 0.9,
                },
            );
        }
    }

    pub fn toggle_mute(&mut self) -> bool {
        self.is_muted = !self.is_muted;
        self.is_muted
    }
}

/// Decodes an Ogg Opus byte stream into a standard 16-bit PCM WAV byte buffer in memory
pub fn decode_opus_to_wav(opus_bytes: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut reader = OggOpusReader::new(Cursor::new(opus_bytes))?;
    let head = reader.head().clone();
    let channels = head.channel_count as usize;
    let sample_rate = 48_000u32;

    let mut decoder = head.decoder(sample_rate as i32)?;
    let mut trim = Trim::new(&head, sample_rate as i32, channels)?;

    let mut block = vec![0.0f32; MAX_PACKET_SAMPLES * channels];
    let mut samples: Vec<i16> = Vec::new();

    for packet in reader.packets() {
        let packet = packet?;
        let n = decoder.decode(&packet.data, MAX_PACKET_SAMPLES, &mut block)?;
        let kept = trim.keep(&packet, &block[..n * channels]);
        for &s in kept {
            let clamped = s.clamp(-1.0, 1.0);
            let s_i16 = (clamped * 32767.0) as i16;
            samples.push(s_i16);
        }
    }

    // Build standard 44-byte RIFF/WAV header
    let num_channels = channels as u16;
    let bits_per_sample = 16u16;
    let bytes_per_sample = 2u16;
    let block_align = num_channels * bytes_per_sample;
    let byte_rate = sample_rate * block_align as u32;
    let data_size = (samples.len() * 2) as u32;
    let file_size = 36 + data_size;

    let mut wav = Vec::with_capacity(44 + data_size as usize);
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&file_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // Subchunk1Size (16 for PCM)
    wav.extend_from_slice(&1u16.to_le_bytes());  // AudioFormat (1 = PCM)
    wav.extend_from_slice(&num_channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());

    for s in samples {
        wav.extend_from_slice(&s.to_le_bytes());
    }

    Ok(wav)
}
