use serde::{Deserialize, Serialize};
use crate::dsp::biquad::{BiquadFilter, BiquadParams, FilterType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoiseSuppressionParams {
    /// 0.0 (Bypass) .. 1.0 (Maximum background separation)
    pub suppression_strength: f32,
    /// Threshold sensitivity in dB (-60.0 .. -20.0 dB)
    pub threshold_db: f32,
    /// Stationary fan/hum cut filter
    pub low_hum_cut: bool,
    /// Fast voice release smoothing
    pub release_ms: f32,
}

impl Default for NoiseSuppressionParams {
    fn default() -> Self {
        Self {
            suppression_strength: 0.65,
            threshold_db: -42.0,
            low_hum_cut: true,
            release_ms: 60.0,
        }
    }
}

/// 4-Band Spectral Subtraction & Noise Separation Filterbank
/// Isolates human speech formants (300Hz - 4kHz) while suppressing stationary air/fan noise,
/// desk rumble, keyboard clatter, and AC line hums.
#[derive(Debug, Clone)]
pub struct NoiseSuppression {
    params: NoiseSuppressionParams,
    sample_rate: f32,
    // 4 crossover filters: Low (<250Hz), Low-Mid (250-1000Hz), High-Mid (1000-4000Hz), High (>4000Hz)
    hum_cut: BiquadFilter,
    band1_lowpass: BiquadFilter,
    band2_bandpass: BiquadFilter,
    band3_bandpass: BiquadFilter,
    band4_highpass: BiquadFilter,
    // Noise floor tracking per band (exponential decay minima)
    noise_floor: [f32; 4],
    band_envelope: [f32; 4],
    band_gain: [f32; 4],
    attack_coeff: f32,
    release_coeff: f32,
}

impl NoiseSuppression {
    pub fn new(sample_rate: f32, params: NoiseSuppressionParams) -> Self {
        let sr = sample_rate.max(8000.0);

        let hc = BiquadFilter::new(sr, BiquadParams {
            filter_type: FilterType::HighPass,
            frequency: 75.0,
            q: 0.707,
            gain_db: 0.0,
        });

        let b1 = BiquadFilter::new(sr, BiquadParams {
            filter_type: FilterType::LowPass,
            frequency: 250.0,
            q: 0.707,
            gain_db: 0.0,
        });

        let b2 = BiquadFilter::new(sr, BiquadParams {
            filter_type: FilterType::BandPass,
            frequency: 625.0,
            q: 1.0,
            gain_db: 0.0,
        });

        let b3 = BiquadFilter::new(sr, BiquadParams {
            filter_type: FilterType::BandPass,
            frequency: 2200.0,
            q: 1.0,
            gain_db: 0.0,
        });

        let b4 = BiquadFilter::new(sr, BiquadParams {
            filter_type: FilterType::HighPass,
            frequency: 4000.0,
            q: 0.707,
            gain_db: 0.0,
        });

        let mut ns = Self {
            params,
            sample_rate: sr,
            hum_cut: hc,
            band1_lowpass: b1,
            band2_bandpass: b2,
            band3_bandpass: b3,
            band4_highpass: b4,
            noise_floor: [0.002, 0.0015, 0.001, 0.0008],
            band_envelope: [0.0; 4],
            band_gain: [1.0; 4],
            attack_coeff: 0.0,
            release_coeff: 0.0,
        };
        ns.recalculate();
        ns
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(8000.0);
        self.hum_cut.set_sample_rate(self.sample_rate);
        self.band1_lowpass.set_sample_rate(self.sample_rate);
        self.band2_bandpass.set_sample_rate(self.sample_rate);
        self.band3_bandpass.set_sample_rate(self.sample_rate);
        self.band4_highpass.set_sample_rate(self.sample_rate);
        self.recalculate();
    }

    pub fn set_params(&mut self, params: NoiseSuppressionParams) {
        self.params = params;
        self.recalculate();
    }

    pub fn params(&self) -> &NoiseSuppressionParams {
        &self.params
    }

    pub fn reset(&mut self) {
        self.hum_cut.reset();
        self.band1_lowpass.reset();
        self.band2_bandpass.reset();
        self.band3_bandpass.reset();
        self.band4_highpass.reset();
        self.noise_floor = [0.002, 0.0015, 0.001, 0.0008];
        self.band_envelope = [0.0; 4];
        self.band_gain = [1.0; 4];
    }

    pub fn recalculate(&mut self) {
        self.attack_coeff = (-1.0 / (0.003 * self.sample_rate)).exp(); // Fast 3ms attack for voice clarity
        let rel_sec = (self.params.release_ms * 0.001).clamp(0.010, 0.500);
        self.release_coeff = (-1.0 / (rel_sec * self.sample_rate)).exp();
    }

    #[inline(always)]
    pub fn process_sample(&mut self, input: f32) -> f32 {
        if self.params.suppression_strength <= 0.01 {
            return input;
        }

        // 1. Rumble & AC Hum Cut
        let clean_input = if self.params.low_hum_cut {
            self.hum_cut.process_sample(input)
        } else {
            input
        };

        // 2. Filter into 4 spectral bands
        let s1 = self.band1_lowpass.process_sample(clean_input);
        let s2 = self.band2_bandpass.process_sample(clean_input);
        let s3 = self.band3_bandpass.process_sample(clean_input);
        let s4 = self.band4_highpass.process_sample(clean_input);

        let bands = [s1, s2, s3, s4];
        let thresh_lin = 10.0_f32.powf(self.params.threshold_db / 20.0);
        let max_attenuation = (self.params.suppression_strength * 0.95).clamp(0.0, 0.95);

        let mut output = 0.0f32;

        for (i, &band_sample) in bands.iter().enumerate() {
            let abs_s = band_sample.abs();

            // Track band envelope with fast attack / gentle release
            if abs_s > self.band_envelope[i] {
                self.band_envelope[i] = self.attack_coeff * self.band_envelope[i] + (1.0 - self.attack_coeff) * abs_s;
            } else {
                self.band_envelope[i] = self.release_coeff * self.band_envelope[i] + (1.0 - self.release_coeff) * abs_s;
            }

            // Adaptive noise floor tracking (minima tracker)
            if self.band_envelope[i] < self.noise_floor[i] {
                self.noise_floor[i] = self.band_envelope[i];
            } else {
                // Very slow drift upward to adapt to new ambient noise (e.g. AC kicking in)
                self.noise_floor[i] += (self.band_envelope[i] - self.noise_floor[i]) * 0.000005;
            }

            // Speech vs Noise SNR ratio
            let noise_threshold = (self.noise_floor[i] * 2.5).max(thresh_lin * 0.2);
            let target_gain = if self.band_envelope[i] > noise_threshold {
                1.0
            } else if self.band_envelope[i] > thresh_lin * 0.05 {
                let snr = (self.band_envelope[i] - thresh_lin * 0.05) / (noise_threshold - thresh_lin * 0.05);
                1.0 - (1.0 - snr) * max_attenuation
            } else {
                1.0 - max_attenuation
            };

            // Smooth gain change
            self.band_gain[i] += (target_gain - self.band_gain[i]) * 0.05;
            output += bands[i] * self.band_gain[i];
        }

        output
    }

    pub fn process_block(&mut self, buffer: &mut [f32]) {
        for s in buffer.iter_mut() {
            *s = self.process_sample(*s);
        }
    }
}
