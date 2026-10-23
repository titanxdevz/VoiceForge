use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FdnReverbParams {
    pub room_size: f32, // 0.1 .. 1.0
    pub damping: f32,   // 0.0 .. 1.0 (high frequency loss)
    pub width: f32,     // 0.0 .. 1.0 (stereo spread)
    pub dry_wet: f32,   // 0.0 .. 1.0
}

impl Default for FdnReverbParams {
    fn default() -> Self {
        Self {
            room_size: 0.7,
            damping: 0.35,
            width: 0.8,
            dry_wet: 0.25,
        }
    }
}

/// 4-Line Feedback Delay Network (FDN) Reverb with Hadamard mixing matrix
#[derive(Debug, Clone)]
pub struct FdnReverb {
    params: FdnReverbParams,
    sample_rate: f32,
    // Delay line buffers
    delay_lines: [Vec<f32>; 4],
    delay_indices: [usize; 4],
    delay_lengths: [usize; 4],
    // Lowpass damping filter state per line
    damping_state: [f32; 4],
}

impl FdnReverb {
    // Prime-based delay lengths at 48kHz for high echo density without flutter
    const BASE_DELAYS_48K: [usize; 4] = [1087, 1321, 1559, 1877];

    pub fn new(sample_rate: f32, params: FdnReverbParams) -> Self {
        let sr = sample_rate.max(8000.0);
        let ratio = sr / 48000.0;

        let d0 = ((Self::BASE_DELAYS_48K[0] as f32 * ratio).round() as usize).max(64);
        let d1 = ((Self::BASE_DELAYS_48K[1] as f32 * ratio).round() as usize).max(64);
        let d2 = ((Self::BASE_DELAYS_48K[2] as f32 * ratio).round() as usize).max(64);
        let d3 = ((Self::BASE_DELAYS_48K[3] as f32 * ratio).round() as usize).max(64);

        Self {
            params,
            sample_rate: sr,
            delay_lines: [
                vec![0.0f32; d0],
                vec![0.0f32; d1],
                vec![0.0f32; d2],
                vec![0.0f32; d3],
            ],
            delay_indices: [0; 4],
            delay_lengths: [d0, d1, d2, d3],
            damping_state: [0.0; 4],
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        *self = Self::new(sample_rate, self.params.clone());
    }

    pub fn set_params(&mut self, params: FdnReverbParams) {
        self.params = params;
    }

    pub fn params(&self) -> &FdnReverbParams {
        &self.params
    }

    pub fn reset(&mut self) {
        for line in self.delay_lines.iter_mut() {
            line.fill(0.0);
        }
        self.damping_state.fill(0.0);
        self.delay_indices.fill(0);
    }

    #[inline(always)]
    pub fn process_sample(&mut self, input: f32) -> (f32, f32) {
        let feedback_gain = self.params.room_size.clamp(0.05, 0.96);
        let damp = self.params.damping.clamp(0.0, 0.95);

        // Read outputs from delay lines
        let mut y = [0.0f32; 4];
        for (i, y_elem) in y.iter_mut().enumerate() {
            *y_elem = self.delay_lines[i][self.delay_indices[i]];
            // One-pole lowpass filter for air absorption / frequency damping
            self.damping_state[i] = *y_elem * (1.0 - damp) + self.damping_state[i] * damp;
            *y_elem = self.damping_state[i];
        }

        // Fast 4x4 orthogonal Householder / Hadamard matrix multiplication
        // [ 1  1  1  1 ]
        // [ 1 -1  1 -1 ] * 0.5
        // [ 1  1 -1 -1 ]
        // [ 1 -1 -1  1 ]
        let u0 = 0.5 * (y[0] + y[1] + y[2] + y[3]);
        let u1 = 0.5 * (y[0] - y[1] + y[2] - y[3]);
        let u2 = 0.5 * (y[0] + y[1] - y[2] - y[3]);
        let u3 = 0.5 * (y[0] - y[1] - y[2] + y[3]);

        // Inject input + feedback into delay lines
        let in_scaled = input * 0.5;
        let next_inputs = [
            in_scaled + u0 * feedback_gain,
            in_scaled + u1 * feedback_gain,
            in_scaled + u2 * feedback_gain,
            in_scaled + u3 * feedback_gain,
        ];

        for (i, &in_val) in next_inputs.iter().enumerate() {
            let mut sample = in_val;
            if sample.is_nan() || sample.is_infinite() {
                sample = 0.0;
            }
            // Anti-denormal
            if sample.abs() < 1e-15 {
                sample = 0.0;
            }
            self.delay_lines[i][self.delay_indices[i]] = sample;
            self.delay_indices[i] = (self.delay_indices[i] + 1) % self.delay_lengths[i];
        }

        // Stereo tap summation
        let wet_l = (y[0] + y[2]) * 0.5;
        let wet_r = (y[1] + y[3]) * 0.5;

        let wet = self.params.dry_wet.clamp(0.0, 1.0);
        let dry = 1.0 - wet;

        let out_l = input * dry + wet_l * wet;
        let out_r = input * dry + wet_r * wet;

        (out_l, out_r)
    }

    /// Mono-in mono-out convenience
    #[inline(always)]
    pub fn process_mono_sample(&mut self, input: f32) -> f32 {
        let (l, r) = self.process_sample(input);
        (l + r) * 0.5
    }

    pub fn process_block(&mut self, buffer: &mut [f32]) {
        for s in buffer.iter_mut() {
            *s = self.process_mono_sample(*s);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fdn_reverb_stability() {
        let mut rev = FdnReverb::new(48000.0, FdnReverbParams::default());
        for i in 0..2048 {
            let input = if i == 0 { 1.0 } else { 0.0 };
            let (l, r) = rev.process_sample(input);
            assert!(!l.is_nan());
            assert!(!r.is_nan());
            assert!(l.abs() <= 2.0);
            assert!(r.abs() <= 2.0);
        }
    }
}
