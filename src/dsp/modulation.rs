use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibratoParams {
    pub rate_hz: f32,   // 0.5 .. 15.0 Hz
    pub depth_ms: f32,  // 0.0 .. 10.0 ms
    pub dry_wet: f32,   // 0.0 .. 1.0
}

impl Default for VibratoParams {
    fn default() -> Self {
        Self {
            rate_hz: 5.5,
            depth_ms: 2.5,
            dry_wet: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Vibrato {
    params: VibratoParams,
    sample_rate: f32,
    buffer: Vec<f32>,
    write_idx: usize,
    phase: f32,
}

impl Vibrato {
    pub fn new(sample_rate: f32, params: VibratoParams) -> Self {
        let sr = sample_rate.max(8000.0);
        let max_samples = (sr * 0.05).ceil() as usize; // 50ms buffer
        Self {
            params,
            sample_rate: sr,
            buffer: vec![0.0f32; max_samples.max(512)],
            write_idx: 0,
            phase: 0.0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        *self = Self::new(sample_rate, self.params.clone());
    }

    pub fn set_params(&mut self, params: VibratoParams) {
        self.params = params;
    }

    pub fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.write_idx = 0;
        self.phase = 0.0;
    }

    #[inline(always)]
    pub fn process_sample(&mut self, input: f32) -> f32 {
        let buf_len = self.buffer.len();
        self.buffer[self.write_idx] = input;

        // LFO
        let phase_inc = (2.0 * PI * self.params.rate_hz) / self.sample_rate;
        self.phase = (self.phase + phase_inc).rem_euclid(2.0 * PI);
        let lfo = self.phase.sin(); // -1.0 .. +1.0

        let depth_samples = (self.params.depth_ms * 0.001 * self.sample_rate).clamp(0.0, (buf_len / 2) as f32);
        let current_delay = depth_samples * (1.0 + lfo) * 0.5 + 4.0; // avoid 0 delay

        let read_pos = (self.write_idx as f32) - current_delay;
        let wrapped_pos = read_pos.rem_euclid(buf_len as f32);
        let idx0 = wrapped_pos as usize % buf_len;
        let idx1 = (idx0 + 1) % buf_len;
        let frac = wrapped_pos - wrapped_pos.floor();

        let delayed = self.buffer[idx0] * (1.0 - frac) + self.buffer[idx1] * frac;
        self.write_idx = (self.write_idx + 1) % buf_len;

        let wet = self.params.dry_wet.clamp(0.0, 1.0);
        input * (1.0 - wet) + delayed * wet
    }

    pub fn process_block(&mut self, buffer: &mut [f32]) {
        for s in buffer.iter_mut() {
            *s = self.process_sample(*s);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RingModParams {
    pub carrier_freq_hz: f32, // 20.0 .. 2000.0 Hz (e.g. 50Hz for Dalek/Robot, 440Hz for Alien)
    pub waveform: RingModWaveform,
    pub dry_wet: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RingModWaveform {
    Sine,
    Square,
    Saw,
    PulseTrain,
}

impl Default for RingModParams {
    fn default() -> Self {
        Self {
            carrier_freq_hz: 60.0,
            waveform: RingModWaveform::Sine,
            dry_wet: 0.8,
        }
    }
}

/// Ring Modulator / Robotizer for Dalek, Robot, Vocoder Robot, Alien effects
#[derive(Debug, Clone)]
pub struct RingModulator {
    params: RingModParams,
    sample_rate: f32,
    phase: f32,
}

impl RingModulator {
    pub fn new(sample_rate: f32, params: RingModParams) -> Self {
        Self {
            params,
            sample_rate: sample_rate.max(8000.0),
            phase: 0.0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(8000.0);
    }

    pub fn set_params(&mut self, params: RingModParams) {
        self.params = params;
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
    }

    #[inline(always)]
    pub fn process_sample(&mut self, input: f32) -> f32 {
        let phase_inc = (2.0 * PI * self.params.carrier_freq_hz) / self.sample_rate;
        self.phase = (self.phase + phase_inc).rem_euclid(2.0 * PI);

        let carrier = match self.params.waveform {
            RingModWaveform::Sine => self.phase.sin(),
            RingModWaveform::Square => if self.phase < PI { 1.0 } else { -1.0 },
            RingModWaveform::Saw => (self.phase / PI) - 1.0,
            RingModWaveform::PulseTrain => {
                if self.phase < (PI * 0.2) { 1.0 } else { -0.2 }
            }
        };

        let modulated = input * carrier;
        let wet = self.params.dry_wet.clamp(0.0, 1.0);
        let out = input * (1.0 - wet) + modulated * wet;

        if out.is_nan() || out.is_infinite() { 0.0 } else { out }
    }

    pub fn process_block(&mut self, buffer: &mut [f32]) {
        for s in buffer.iter_mut() {
            *s = self.process_sample(*s);
        }
    }
}
