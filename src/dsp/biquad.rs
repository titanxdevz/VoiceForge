use std::f32::consts::PI;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum FilterType {
    LowPass,
    HighPass,
    BandPass,
    Notch,
    PeakingEQ,
    LowShelf,
    HighShelf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiquadParams {
    pub filter_type: FilterType,
    pub frequency: f32, // Hz (20.0 .. 20000.0)
    pub q: f32,         // Quality factor (0.1 .. 10.0)
    pub gain_db: f32,   // dB (-24.0 .. +24.0)
}

impl Default for BiquadParams {
    fn default() -> Self {
        Self {
            filter_type: FilterType::PeakingEQ,
            frequency: 1000.0,
            q: 0.707,
            gain_db: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BiquadFilter {
    params: BiquadParams,
    sample_rate: f32,
    // Coefficients
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    // State variables (Direct Form II Transposed)
    s1: f32,
    s2: f32,
}

impl BiquadFilter {
    pub fn new(sample_rate: f32, params: BiquadParams) -> Self {
        let mut filter = Self {
            params,
            sample_rate: sample_rate.max(8000.0),
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            s1: 0.0,
            s2: 0.0,
        };
        filter.recalculate();
        filter
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(8000.0);
        self.recalculate();
    }

    pub fn set_params(&mut self, params: BiquadParams) {
        self.params = params;
        self.recalculate();
    }

    pub fn params(&self) -> &BiquadParams {
        &self.params
    }

    pub fn reset(&mut self) {
        self.s1 = 0.0;
        self.s2 = 0.0;
    }

    pub fn recalculate(&mut self) {
        let nyquist = self.sample_rate * 0.499;
        let freq = self.params.frequency.clamp(20.0, nyquist);
        let q = self.params.q.clamp(0.05, 30.0);
        let gain_linear = 10.0_f32.powf(self.params.gain_db / 40.0); // A = 10^(dB/40) for shelves & peaking
        let omega = 2.0 * PI * (freq / self.sample_rate);
        let sn = omega.sin();
        let cs = omega.cos();
        let alpha = sn / (2.0 * q);

        let (b0, b1, b2, a0, a1, a2) = match self.params.filter_type {
            FilterType::LowPass => {
                let b1 = 1.0 - cs;
                let b0 = b1 * 0.5;
                let b2 = b0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cs;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::HighPass => {
                let b1 = -(1.0 + cs);
                let b0 = (1.0 + cs) * 0.5;
                let b2 = b0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cs;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::BandPass => {
                let b0 = alpha;
                let b1 = 0.0;
                let b2 = -alpha;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cs;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::Notch => {
                let b0 = 1.0;
                let b1 = -2.0 * cs;
                let b2 = 1.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cs;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::PeakingEQ => {
                let a = gain_linear;
                let b0 = 1.0 + alpha * a;
                let b1 = -2.0 * cs;
                let b2 = 1.0 - alpha * a;
                let a0 = 1.0 + alpha / a;
                let a1 = -2.0 * cs;
                let a2 = 1.0 - alpha / a;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::LowShelf => {
                let a = gain_linear;
                let sqrt_a = a.sqrt();
                let b0 = a * ((a + 1.0) - (a - 1.0) * cs + 2.0 * sqrt_a * alpha);
                let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cs);
                let b2 = a * ((a + 1.0) - (a - 1.0) * cs - 2.0 * sqrt_a * alpha);
                let a0 = (a + 1.0) + (a - 1.0) * cs + 2.0 * sqrt_a * alpha;
                let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cs);
                let a2 = (a + 1.0) + (a - 1.0) * cs - 2.0 * sqrt_a * alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::HighShelf => {
                let a = gain_linear;
                let sqrt_a = a.sqrt();
                let b0 = a * ((a + 1.0) + (a - 1.0) * cs + 2.0 * sqrt_a * alpha);
                let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cs);
                let b2 = a * ((a + 1.0) + (a - 1.0) * cs - 2.0 * sqrt_a * alpha);
                let a0 = (a + 1.0) - (a - 1.0) * cs + 2.0 * sqrt_a * alpha;
                let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cs);
                let a2 = (a + 1.0) - (a - 1.0) * cs - 2.0 * sqrt_a * alpha;
                (b0, b1, b2, a0, a1, a2)
            }
        };

        let inv_a0 = if a0.abs() > 1e-9 { 1.0 / a0 } else { 1.0 };
        self.b0 = b0 * inv_a0;
        self.b1 = b1 * inv_a0;
        self.b2 = b2 * inv_a0;
        self.a1 = a1 * inv_a0;
        self.a2 = a2 * inv_a0;
    }

    #[inline(always)]
    pub fn process_sample(&mut self, input: f32) -> f32 {
        // Direct Form II Transposed:
        // y[n] = b0 * x[n] + s1
        // s1   = b1 * x[n] - a1 * y[n] + s2
        // s2   = b2 * x[n] - a2 * y[n]
        let out = self.b0 * input + self.s1;
        self.s1 = self.b1 * input - self.a1 * out + self.s2;
        self.s2 = self.b2 * input - self.a2 * out;

        // Anti-denormal prevention
        if self.s1.abs() < 1e-15 { self.s1 = 0.0; }
        if self.s2.abs() < 1e-15 { self.s2 = 0.0; }

        if out.is_nan() || out.is_infinite() {
            self.reset();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_biquad_stability_and_no_nan() {
        let mut filter = BiquadFilter::new(48000.0, BiquadParams {
            filter_type: FilterType::LowPass,
            frequency: 1000.0,
            q: 0.707,
            gain_db: 0.0,
        });

        for i in 0..1000 {
            let input = if i == 0 { 1.0 } else { 0.0 }; // impulse
            let out = filter.process_sample(input);
            assert!(!out.is_nan(), "Filter output must not be NaN");
            assert!(out.abs() <= 2.0, "Filter output must be bounded");
        }
    }

    #[test]
    fn test_shelving_filters() {
        let mut shelf = BiquadFilter::new(48000.0, BiquadParams {
            filter_type: FilterType::LowShelf,
            frequency: 150.0,
            q: 0.707,
            gain_db: 6.0,
        });
        let mut block = [0.1f32; 128];
        shelf.process_block(&mut block);
        for &sample in &block {
            assert!(!sample.is_nan());
        }
    }
}
