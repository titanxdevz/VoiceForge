use serde::{Deserialize, Serialize};
use crate::presets::VoicePreset;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioCommand {
    Start,
    Stop,
    SetPreset(VoicePreset),
    SetBypass(bool),
    SetMasterGain(f32),
    SetPitchFormant { pitch: f32, formant: f32 },
    SetMonitoring(bool),
    PlaySoundboardSample(Vec<f32>),
    StopSoundboard,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct AudioTelemetry {
    pub input_peak_db: f32,
    pub output_peak_db: f32,
    pub input_rms_db: f32,
    pub output_rms_db: f32,
    pub estimated_latency_ms: f32,
    pub cpu_load_pct: f32,
    pub dropouts: u64,
    pub is_running: bool,
    pub sample_rate: u32,
    pub buffer_size: usize,
}
