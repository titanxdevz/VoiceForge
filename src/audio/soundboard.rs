use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundboardClip {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub hotkey: Option<String>,
    pub volume: f32, // 0.0 .. 1.0
}

pub struct SoundboardEngine;

impl SoundboardEngine {
    /// Decode an audio file (WAV, MP3, OGG, FLAC) into 32-bit float mono samples at target sample rate
    pub fn load_audio_file(path: &Path, target_sample_rate: u32) -> Result<Vec<f32>, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let reader = BufReader::new(file);

        let decoder = rodio::Decoder::new(reader)
            .map_err(|e| format!("Audio decoding failed: {}", e))?;

        use rodio::Source;
        let original_sample_rate = decoder.sample_rate().get();
        let channels = decoder.channels().get() as usize;

        // Collect raw interleaved samples (decoder already yields f32)
        let samples: Vec<f32> = decoder.collect();
        if samples.is_empty() {
            return Err("Audio file contains no samples".to_string());
        }

        // Convert multi-channel to mono
        let mut mono_samples = Vec::with_capacity(samples.len() / channels);
        for chunk in samples.chunks(channels) {
            let sum: f32 = chunk.iter().sum();
            mono_samples.push(sum / (channels as f32));
        }

        // Resample if necessary (linear interpolation)
        if original_sample_rate == target_sample_rate {
            Ok(mono_samples)
        } else {
            let ratio = original_sample_rate as f64 / target_sample_rate as f64;
            let output_len = ((mono_samples.len() as f64) / ratio).floor() as usize;
            let mut resampled = Vec::with_capacity(output_len);

            for i in 0..output_len {
                let src_pos = i as f64 * ratio;
                let idx0 = src_pos.floor() as usize;
                let idx1 = (idx0 + 1).min(mono_samples.len() - 1);
                let frac = (src_pos - idx0 as f64) as f32;

                let s = mono_samples[idx0] * (1.0 - frac) + mono_samples[idx1] * frac;
                resampled.push(s);
            }

            Ok(resampled)
        }
    }
}
