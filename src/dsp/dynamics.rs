use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressorParams {
    pub threshold_db: f32, // -60.0 .. 0.0 dB
    pub ratio: f32,        // 1.0 .. 20.0
    pub attack_ms: f32,    // 0.1 .. 100.0 ms
    pub release_ms: f32,   // 5.0 .. 1000.0 ms
    pub makeup_gain_db: f32, // 0.0 .. 24.0 dB
    pub knee_db: f32,      // 0.0 .. 12.0 dB
}

impl Default for CompressorParams {
    fn default() -> Self {
        Self {
            threshold_db: -18.0,
            ratio: 3.5,
            attack_ms: 10.0,
            release_ms: 120.0,
            makeup_gain_db: 3.0,
            knee_db: 4.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Compressor {
    params: CompressorParams,
    sample_rate: f32,
    envelope: f32,
    attack_coeff: f32,
    release_coeff: f32,
    makeup_gain: f32,
}

impl Compressor {
    pub fn new(sample_rate: f32, params: CompressorParams) -> Self {
        let mut comp = Self {
            params,
            sample_rate: sample_rate.max(8000.0),
            envelope: 0.0,
            attack_coeff: 0.0,
            release_coeff: 0.0,
            makeup_gain: 1.0,
        };
        comp.recalculate();
        comp
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(8000.0);
        self.recalculate();
    }

    pub fn set_params(&mut self, params: CompressorParams) {
        self.params = params;
        self.recalculate();
    }

    pub fn params(&self) -> &CompressorParams {
        &self.params
    }

    pub fn reset(&mut self) {
        self.envelope = 0.0;
    }

    pub fn recalculate(&mut self) {
        let att_sec = (self.params.attack_ms * 0.001).max(0.0001);
        let rel_sec = (self.params.release_ms * 0.001).max(0.001);
        self.attack_coeff = (-1.0 / (att_sec * self.sample_rate)).exp();
        self.release_coeff = (-1.0 / (rel_sec * self.sample_rate)).exp();
        self.makeup_gain = 10.0_f32.powf(self.params.makeup_gain_db / 20.0);
    }

    #[inline(always)]
    pub fn process_sample(&mut self, input: f32) -> f32 {
        let input_abs = input.abs();
        
        // Envelope detection (smooth peak)
        if input_abs > self.envelope {
            self.envelope = self.attack_coeff * self.envelope + (1.0 - self.attack_coeff) * input_abs;
        } else {
            self.envelope = self.release_coeff * self.envelope + (1.0 - self.release_coeff) * input_abs;
        }

        // Convert envelope to dB
        let env_db = if self.envelope > 1e-6 {
            20.0 * self.envelope.log10()
        } else {
            -120.0
        };

        // Soft-knee gain calculation
        let thresh = self.params.threshold_db;
        let knee = self.params.knee_db.max(0.001);
        let half_knee = knee * 0.5;
        let ratio = self.params.ratio.max(1.0);

        let gain_reduction_db = if env_db <= thresh - half_knee {
            0.0
        } else if env_db > thresh + half_knee {
            (env_db - thresh) * (1.0 - 1.0 / ratio)
        } else {
            let excess = env_db - thresh + half_knee;
            (1.0 - 1.0 / ratio) * (excess * excess) / (2.0 * knee)
        };

        let gain = 10.0_f32.powf(-gain_reduction_db / 20.0) * self.makeup_gain;
        let out = input * gain;

        if out.is_nan() || out.is_infinite() {
            0.0
        } else {
            out
        }
    }

    pub fn process_block(&mut self, buffer: &mut [f32]) {
        for s in buffer.iter_mut() {
            *s = self.process_sample(*s);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimiterParams {
    pub ceiling_db: f32, // e.g. -0.5 dB
    pub release_ms: f32, // e.g. 50.0 ms
}

impl Default for LimiterParams {
    fn default() -> Self {
        Self {
            ceiling_db: -0.5,
            release_ms: 50.0,
        }
    }
}

/// Fast lookahead-free brickwall soft-limiter to guarantee zero clipping
#[derive(Debug, Clone)]
pub struct Limiter {
    params: LimiterParams,
    sample_rate: f32,
    envelope: f32,
    release_coeff: f32,
    ceiling_linear: f32,
}

impl Limiter {
    pub fn new(sample_rate: f32, params: LimiterParams) -> Self {
        let mut lim = Self {
            params,
            sample_rate: sample_rate.max(8000.0),
            envelope: 0.0,
            release_coeff: 0.0,
            ceiling_linear: 1.0,
        };
        lim.recalculate();
        lim
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(8000.0);
        self.recalculate();
    }

    pub fn set_params(&mut self, params: LimiterParams) {
        self.params = params;
        self.recalculate();
    }

    pub fn recalculate(&mut self) {
        let rel_sec = (self.params.release_ms * 0.001).max(0.001);
        self.release_coeff = (-1.0 / (rel_sec * self.sample_rate)).exp();
        self.ceiling_linear = 10.0_f32.powf(self.params.ceiling_db / 20.0).clamp(0.01, 1.0);
    }

    pub fn reset(&mut self) {
        self.envelope = 0.0;
    }

    #[inline(always)]
    pub fn process_sample(&mut self, input: f32) -> f32 {
        let input_abs = input.abs();
        if input_abs > self.envelope {
            self.envelope = input_abs;
        } else {
            self.envelope = self.release_coeff * self.envelope + (1.0 - self.release_coeff) * input_abs;
        }

        let gain = if self.envelope > self.ceiling_linear {
            self.ceiling_linear / self.envelope
        } else {
            1.0
        };

        let limited = input * gain;
        // Hyperbolic tangent soft saturation as extra safety guard against inter-sample peaks
        let normalized = limited / self.ceiling_linear;
        let shaped = if normalized.abs() > 0.95 {
            (normalized * 0.95).tanh() * self.ceiling_linear
        } else {
            limited
        };

        if shaped.is_nan() || shaped.is_infinite() {
            0.0
        } else {
            shaped.clamp(-self.ceiling_linear, self.ceiling_linear)
        }
    }

    pub fn process_block(&mut self, buffer: &mut [f32]) {
        for s in buffer.iter_mut() {
            *s = self.process_sample(*s);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoiseGateParams {
    pub threshold_db: f32, // -70.0 .. -20.0 dB
    pub ratio: f32,        // 2.0 .. 20.0 (downward expander)
    pub attack_ms: f32,    // 1.0 .. 50.0 ms
    pub release_ms: f32,   // 10.0 .. 500.0 ms
    pub hold_ms: f32,      // 0.0 .. 200.0 ms
}

impl Default for NoiseGateParams {
    fn default() -> Self {
        Self {
            threshold_db: -45.0,
            ratio: 8.0,
            attack_ms: 2.0,
            release_ms: 100.0,
            hold_ms: 20.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NoiseGate {
    params: NoiseGateParams,
    sample_rate: f32,
    envelope: f32,
    hold_counter: usize,
    hold_samples: usize,
    attack_coeff: f32,
    release_coeff: f32,
    gain_state: f32,
}

impl NoiseGate {
    pub fn new(sample_rate: f32, params: NoiseGateParams) -> Self {
        let mut gate = Self {
            params,
            sample_rate: sample_rate.max(8000.0),
            envelope: 0.0,
            hold_counter: 0,
            hold_samples: 0,
            attack_coeff: 0.0,
            release_coeff: 0.0,
            gain_state: 1.0,
        };
        gate.recalculate();
        gate
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(8000.0);
        self.recalculate();
    }

    pub fn set_params(&mut self, params: NoiseGateParams) {
        self.params = params;
        self.recalculate();
    }

    pub fn params(&self) -> &NoiseGateParams {
        &self.params
    }

    pub fn reset(&mut self) {
        self.envelope = 0.0;
        self.hold_counter = 0;
        self.gain_state = 1.0;
    }

    pub fn recalculate(&mut self) {
        let att_sec = (self.params.attack_ms * 0.001).max(0.0001);
        let rel_sec = (self.params.release_ms * 0.001).max(0.001);
        self.attack_coeff = (-1.0 / (att_sec * self.sample_rate)).exp();
        self.release_coeff = (-1.0 / (rel_sec * self.sample_rate)).exp();
        self.hold_samples = ((self.params.hold_ms * 0.001) * self.sample_rate) as usize;
    }

    #[inline(always)]
    pub fn process_sample(&mut self, input: f32) -> f32 {
        let input_abs = input.abs();
        if input_abs > self.envelope {
            self.envelope = 0.9 * self.envelope + 0.1 * input_abs;
        } else {
            self.envelope = 0.999 * self.envelope + 0.001 * input_abs;
        }

        let env_db = if self.envelope > 1e-6 {
            20.0 * self.envelope.log10()
        } else {
            -120.0
        };

        let target_gain = if env_db >= self.params.threshold_db {
            self.hold_counter = self.hold_samples;
            1.0
        } else if self.hold_counter > 0 {
            self.hold_counter -= 1;
            1.0
        } else {
            let under_db = self.params.threshold_db - env_db;
            let att_db = (under_db * (self.params.ratio - 1.0)).min(60.0);
            10.0_f32.powf(-att_db / 20.0)
        };

        // Smooth gain transition
        if target_gain > self.gain_state {
            self.gain_state = self.attack_coeff * self.gain_state + (1.0 - self.attack_coeff) * target_gain;
        } else {
            self.gain_state = self.release_coeff * self.gain_state + (1.0 - self.release_coeff) * target_gain;
        }

        let out = input * self.gain_state;
        if out.is_nan() || out.is_infinite() { 0.0 } else { out }
    }

    pub fn process_block(&mut self, buffer: &mut [f32]) {
        for s in buffer.iter_mut() {
            *s = self.process_sample(*s);
        }
    }
}

/// De-Esser: side-chain bandpass at sibilance frequencies (5 kHz - 9 kHz) driving dynamic gain reduction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeEsserParams {
    pub frequency: f32,     // 4000.0 .. 9000.0 Hz
    pub threshold_db: f32,  // -40.0 .. 0.0 dB
    pub reduction_db: f32,  // 0.0 .. 18.0 dB
}

impl Default for DeEsserParams {
    fn default() -> Self {
        Self {
            frequency: 6500.0,
            threshold_db: -24.0,
            reduction_db: 8.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeEsser {
    params: DeEsserParams,
    sample_rate: f32,
    bandpass: crate::dsp::biquad::BiquadFilter,
    envelope: f32,
    coeff: f32,
}

impl DeEsser {
    pub fn new(sample_rate: f32, params: DeEsserParams) -> Self {
        let bp = crate::dsp::biquad::BiquadFilter::new(
            sample_rate,
            crate::dsp::biquad::BiquadParams {
                filter_type: crate::dsp::biquad::FilterType::BandPass,
                frequency: params.frequency,
                q: 2.0,
                gain_db: 0.0,
            },
        );
        let mut deesser = Self {
            params,
            sample_rate: sample_rate.max(8000.0),
            bandpass: bp,
            envelope: 0.0,
            coeff: 0.0,
        };
        deesser.recalculate();
        deesser
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(8000.0);
        self.bandpass.set_sample_rate(self.sample_rate);
        self.recalculate();
    }

    pub fn set_params(&mut self, params: DeEsserParams) {
        self.params = params;
        self.bandpass.set_params(crate::dsp::biquad::BiquadParams {
            filter_type: crate::dsp::biquad::FilterType::BandPass,
            frequency: self.params.frequency,
            q: 2.0,
            gain_db: 0.0,
        });
        self.recalculate();
    }

    pub fn reset(&mut self) {
        self.bandpass.reset();
        self.envelope = 0.0;
    }

    pub fn recalculate(&mut self) {
        self.coeff = (-1.0 / (0.010 * self.sample_rate)).exp(); // 10ms smooth
    }

    #[inline(always)]
    pub fn process_sample(&mut self, input: f32) -> f32 {
        let sibilant = self.bandpass.process_sample(input).abs();
        if sibilant > self.envelope {
            self.envelope = 0.8 * self.envelope + 0.2 * sibilant;
        } else {
            self.envelope = self.coeff * self.envelope + (1.0 - self.coeff) * sibilant;
        }

        let env_db = if self.envelope > 1e-6 {
            20.0 * self.envelope.log10()
        } else {
            -120.0
        };

        let gain = if env_db > self.params.threshold_db {
            let excess = env_db - self.params.threshold_db;
            let att = (excess * 0.7).min(self.params.reduction_db);
            10.0_f32.powf(-att / 20.0)
        } else {
            1.0
        };

        input * gain
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
    fn test_limiter_brickwall_guarantee() {
        let mut limiter = Limiter::new(48000.0, LimiterParams {
            ceiling_db: -0.5,
            release_ms: 10.0,
        });
        let ceiling = 10.0_f32.powf(-0.5 / 20.0);

        // Huge blast of loud audio (+18 dB = 8.0 amplitude)
        for _ in 0..500 {
            let out = limiter.process_sample(8.0);
            assert!(!out.is_nan());
            assert!(out.abs() <= ceiling + 1e-4, "Out {} must not exceed ceiling {}", out, ceiling);
        }
    }

    #[test]
    fn test_compressor_smoothness() {
        let mut comp = Compressor::new(48000.0, CompressorParams::default());
        let mut signal = [0.5f32; 256];
        comp.process_block(&mut signal);
        for &s in &signal {
            assert!(!s.is_nan());
            assert!(s.is_finite());
        }
    }
}
