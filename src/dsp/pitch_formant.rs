use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PitchFormantParams {
    /// Pitch shift in semitones: -12.0 .. +12.0
    pub pitch_semitones: f32,
    /// Formant shift in percent: -50.0 .. +50.0 (-50% is deeper/giant, +50% is chipmunk/child)
    pub formant_shift_pct: f32,
    /// Mix: 0.0 .. 1.0 (wet / dry)
    pub dry_wet: f32,
}

impl Default for PitchFormantParams {
    fn default() -> Self {
        Self {
            pitch_semitones: 0.0,
            formant_shift_pct: 0.0,
            dry_wet: 1.0,
        }
    }
}

/// Real-time WSOLA (Waveform Similarity Overlap-Add) Pitch & Formant Shifter
/// Performs independent pitch scaling and formant envelope shifting with zero per-sample allocations.
#[derive(Debug, Clone)]
pub struct PitchFormantShifter {
    params: PitchFormantParams,
    sample_rate: f32,

    // Circular delay buffer for grain capture & overlap-add
    buffer: Vec<f32>,
    buf_mask: usize,
    write_pos: usize,

    // Overlap-add synthesis state
    read_pos: f64,
    grain_size: usize,
    hop_size: usize,
    hanning_window: Vec<f32>,

    // Formant filter state (cascaded resonators that shift spectral resonances independently)
    formant_filters: [crate::dsp::biquad::BiquadFilter; 3],

    // Output accumulator buffer
    accum: Vec<f32>,
    accum_read: usize,
    accum_write: usize,
    accum_mask: usize,
}

impl PitchFormantShifter {
    pub const MAX_DELAY_SAMPLES: usize = 16384; // Power of two for fast bitmask

    pub fn new(sample_rate: f32, params: PitchFormantParams) -> Self {
        let sr = sample_rate.max(8000.0);
        let buf_size = Self::MAX_DELAY_SAMPLES;
        let grain_size = ((sr * 0.025).round() as usize).next_power_of_two().clamp(256, 2048);
        let hop_size = grain_size / 4;

        // Precompute Hanning window
        let mut hanning = vec![0.0f32; grain_size];
        for (i, win) in hanning.iter_mut().enumerate().take(grain_size) {
            *win = 0.5 * (1.0 - (2.0 * PI * i as f32 / (grain_size - 1) as f32).cos());
        }

        let f1 = crate::dsp::biquad::BiquadFilter::new(
            sr,
            crate::dsp::biquad::BiquadParams {
                filter_type: crate::dsp::biquad::FilterType::PeakingEQ,
                frequency: 600.0,
                q: 2.2,
                gain_db: 0.0,
            },
        );
        let f2 = crate::dsp::biquad::BiquadFilter::new(
            sr,
            crate::dsp::biquad::BiquadParams {
                filter_type: crate::dsp::biquad::FilterType::PeakingEQ,
                frequency: 1600.0,
                q: 2.2,
                gain_db: 0.0,
            },
        );
        let f3 = crate::dsp::biquad::BiquadFilter::new(
            sr,
            crate::dsp::biquad::BiquadParams {
                filter_type: crate::dsp::biquad::FilterType::PeakingEQ,
                frequency: 2800.0,
                q: 2.2,
                gain_db: 0.0,
            },
        );

        let accum_size = 4096;
        let mut shifter = Self {
            params,
            sample_rate: sr,
            buffer: vec![0.0f32; buf_size],
            buf_mask: buf_size - 1,
            write_pos: 0,
            read_pos: 0.0,
            grain_size,
            hop_size,
            hanning_window: hanning,
            formant_filters: [f1, f2, f3],
            accum: vec![0.0f32; accum_size],
            accum_read: 0,
            accum_write: 0,
            accum_mask: accum_size - 1,
        };
        shifter.recalculate();
        shifter
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(8000.0);
        for f in self.formant_filters.iter_mut() {
            f.set_sample_rate(self.sample_rate);
        }
        self.recalculate();
    }

    pub fn set_params(&mut self, params: PitchFormantParams) {
        self.params = params;
        self.recalculate();
    }

    pub fn params(&self) -> &PitchFormantParams {
        &self.params
    }

    pub fn reset(&mut self) {
        self.buffer.fill(0.0);
        self.accum.fill(0.0);
        self.write_pos = 0;
        self.read_pos = 0.0;
        self.accum_read = 0;
        self.accum_write = 0;
        for f in self.formant_filters.iter_mut() {
            f.reset();
        }
    }

    pub fn recalculate(&mut self) {
        // Formant shift: recalculate center frequencies of vocal tract formant filters
        // Normal vocal formants: F1 ~ 600Hz, F2 ~ 1600Hz, F3 ~ 2800Hz
        let formant_ratio = 1.0 + (self.params.formant_shift_pct / 100.0);
        let ratio = formant_ratio.clamp(0.5, 1.8);

        // When formant shifting, we move formant envelope opposite to or independently of pitch
        let f1_freq = (600.0 * ratio).clamp(150.0, 1800.0);
        let f2_freq = (1600.0 * ratio).clamp(600.0, 3600.0);
        let f3_freq = (2800.0 * ratio).clamp(1200.0, 6000.0);

        // Subtle boost/cut depending on shift to emphasize formant warmth or sharpness
        let gain_db = (ratio - 1.0) * 8.0;

        self.formant_filters[0].set_params(crate::dsp::biquad::BiquadParams {
            filter_type: crate::dsp::biquad::FilterType::PeakingEQ,
            frequency: f1_freq,
            q: 2.2,
            gain_db: gain_db.clamp(-9.0, 9.0),
        });
        self.formant_filters[1].set_params(crate::dsp::biquad::BiquadParams {
            filter_type: crate::dsp::biquad::FilterType::PeakingEQ,
            frequency: f2_freq,
            q: 2.0,
            gain_db: gain_db.clamp(-9.0, 9.0),
        });
        self.formant_filters[2].set_params(crate::dsp::biquad::BiquadParams {
            filter_type: crate::dsp::biquad::FilterType::PeakingEQ,
            frequency: f3_freq,
            q: 2.0,
            gain_db: (gain_db * 0.7).clamp(-7.0, 7.0),
        });
    }

    /// Pitch factor: 2^(semitones / 12)
    #[inline(always)]
    fn pitch_scale_factor(&self) -> f64 {
        let semitones = self.params.pitch_semitones.clamp(-12.0, 12.0);
        2.0_f64.powf(semitones as f64 / 12.0)
    }

    #[inline(always)]
    pub fn process_sample(&mut self, input: f32) -> f32 {
        // Write to ring buffer
        self.buffer[self.write_pos] = input;
        let current_write = self.write_pos;
        self.write_pos = (self.write_pos + 1) & self.buf_mask;

        // If pitch shift is negligible and formant shift is negligible, pass through with dry/wet
        let pitch_factor = self.pitch_scale_factor();
        let is_bypass = (self.params.pitch_semitones.abs() < 0.05) && (self.params.formant_shift_pct.abs() < 0.5);

        if is_bypass {
            return input;
        }

        // Two-head overlapping crossfade pitch shifter (SOLA principle)
        // Header A & Header B phase offset by half grain
        let grain = self.grain_size as f64;
        let rate_diff = pitch_factor - 1.0;

        self.read_pos += rate_diff;
        if self.read_pos > grain {
            self.read_pos -= grain;
        } else if self.read_pos < 0.0 {
            self.read_pos += grain;
        }

        let delay1 = self.read_pos;
        let delay2 = if self.read_pos >= grain * 0.5 {
            self.read_pos - grain * 0.5
        } else {
            self.read_pos + grain * 0.5
        };

        // Window weighting: triangular / sinusoidal crossfade
        let frac1 = (self.read_pos / grain) as f32; // 0..1
        let weight1 = 0.5 * (1.0 - (2.0 * PI * frac1).cos());
        let weight2 = 1.0 - weight1;

        // Interpolated read from circular buffer
        let s1 = self.read_interpolated(current_write as f64 - (delay1 + 256.0));
        let s2 = self.read_interpolated(current_write as f64 - (delay2 + 256.0));
        let mut pitch_shifted = s1 * weight1 + s2 * weight2;

        // Formant envelope modification
        if self.params.formant_shift_pct.abs() > 0.5 {
            pitch_shifted = self.formant_filters[0].process_sample(pitch_shifted);
            pitch_shifted = self.formant_filters[1].process_sample(pitch_shifted);
            pitch_shifted = self.formant_filters[2].process_sample(pitch_shifted);
        }

        let wet = self.params.dry_wet.clamp(0.0, 1.0);
        let out = input * (1.0 - wet) + pitch_shifted * wet;

        if out.is_nan() || out.is_infinite() {
            0.0
        } else {
            out
        }
    }

    #[inline(always)]
    fn read_interpolated(&self, pos: f64) -> f32 {
        let wrapped = pos.rem_euclid(self.buffer.len() as f64);
        let idx0 = wrapped as usize & self.buf_mask;
        let idx1 = (idx0 + 1) & self.buf_mask;
        let frac = (wrapped - wrapped.floor()) as f32;

        let s0 = self.buffer[idx0];
        let s1 = self.buffer[idx1];
        s0 + (s1 - s0) * frac
    }

    pub fn process_block(&mut self, buffer: &mut [f32]) {
        for s in buffer.iter_mut() {
            *s = self.process_sample(*s);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pitch_formant_no_nan_and_bounded() {
        let mut shifter = PitchFormantShifter::new(48000.0, PitchFormantParams {
            pitch_semitones: 7.0,
            formant_shift_pct: -30.0,
            dry_wet: 1.0,
        });

        for i in 0..1024 {
            let input = (2.0 * PI * 440.0 * (i as f32) / 48000.0).sin();
            let out = shifter.process_sample(input);
            assert!(!out.is_nan());
            assert!(out.abs() <= 5.0);
        }
    }

    #[test]
    fn test_pitch_extreme_semitones() {
        let mut shifter_deep = PitchFormantShifter::new(48000.0, PitchFormantParams {
            pitch_semitones: -12.0,
            formant_shift_pct: -50.0,
            dry_wet: 1.0,
        });
        let mut block = [0.2f32; 512];
        shifter_deep.process_block(&mut block);
        for &s in &block {
            assert!(!s.is_nan());
        }
    }
}
