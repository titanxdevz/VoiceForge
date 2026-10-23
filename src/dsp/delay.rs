use serde::{Deserialize, Serialize};
use crate::dsp::biquad::{BiquadFilter, BiquadParams, FilterType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EchoDelayParams {
    pub delay_time_ms: f32, // 20.0 .. 1000.0 ms
    pub feedback: f32,      // 0.0 .. 0.95
    pub damping_hz: f32,    // 500.0 .. 16000.0 Hz (lowpass filter on feedback)
    pub ping_pong: bool,    // Alternating stereo spatial bounce
    pub dry_wet: f32,       // 0.0 .. 1.0
}

impl Default for EchoDelayParams {
    fn default() -> Self {
        Self {
            delay_time_ms: 280.0,
            feedback: 0.42,
            damping_hz: 4500.0,
            ping_pong: true,
            dry_wet: 0.35,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EchoDelay {
    params: EchoDelayParams,
    sample_rate: f32,
    buffer_left: Vec<f32>,
    buffer_right: Vec<f32>,
    write_pos: usize,
    buffer_len: usize,
    damping_filter_l: BiquadFilter,
    damping_filter_r: BiquadFilter,
    current_delay_samples: f32,
    target_delay_samples: f32,
}

impl EchoDelay {
    pub fn new(sample_rate: f32, params: EchoDelayParams) -> Self {
        let sr = sample_rate.max(8000.0);
        // Max 2.0 seconds buffer
        let max_samples = (sr * 2.0) as usize;
        let mut d_l = BiquadFilter::new(sr, BiquadParams {
            filter_type: FilterType::LowPass,
            frequency: params.damping_hz,
            q: 0.707,
            gain_db: 0.0,
        });
        let mut d_r = BiquadFilter::new(sr, BiquadParams {
            filter_type: FilterType::LowPass,
            frequency: params.damping_hz,
            q: 0.707,
            gain_db: 0.0,
        });
        d_l.recalculate();
        d_r.recalculate();

        let target_samples = (params.delay_time_ms * 0.001 * sr).clamp(1.0, max_samples as f32 - 1.0);

        Self {
            params,
            sample_rate: sr,
            buffer_left: vec![0.0; max_samples],
            buffer_right: vec![0.0; max_samples],
            write_pos: 0,
            buffer_len: max_samples,
            damping_filter_l: d_l,
            damping_filter_r: d_r,
            current_delay_samples: target_samples,
            target_delay_samples: target_samples,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(8000.0);
        let max_samples = (self.sample_rate * 2.0) as usize;
        self.buffer_left.resize(max_samples, 0.0);
        self.buffer_right.resize(max_samples, 0.0);
        self.buffer_len = max_samples;
        self.damping_filter_l.set_sample_rate(self.sample_rate);
        self.damping_filter_r.set_sample_rate(self.sample_rate);
        self.recalculate();
    }

    pub fn set_params(&mut self, params: EchoDelayParams) {
        self.params = params;
        self.recalculate();
    }

    pub fn params(&self) -> &EchoDelayParams {
        &self.params
    }

    pub fn reset(&mut self) {
        self.buffer_left.fill(0.0);
        self.buffer_right.fill(0.0);
        self.write_pos = 0;
        self.damping_filter_l.reset();
        self.damping_filter_r.reset();
    }

    pub fn recalculate(&mut self) {
        self.target_delay_samples = (self.params.delay_time_ms * 0.001 * self.sample_rate)
            .clamp(1.0, self.buffer_len as f32 - 2.0);

        self.damping_filter_l.set_params(BiquadParams {
            filter_type: FilterType::LowPass,
            frequency: self.params.damping_hz.clamp(200.0, self.sample_rate * 0.48),
            q: 0.707,
            gain_db: 0.0,
        });
        self.damping_filter_r.set_params(BiquadParams {
            filter_type: FilterType::LowPass,
            frequency: self.params.damping_hz.clamp(200.0, self.sample_rate * 0.48),
            q: 0.707,
            gain_db: 0.0,
        });
    }

    #[inline]
    fn read_interpolated(&self, buffer: &[f32], delay_samples: f32) -> f32 {
        let read_f = (self.write_pos as f32 + self.buffer_len as f32 - delay_samples) % (self.buffer_len as f32);
        let idx0 = read_f.floor() as usize % self.buffer_len;
        let idx1 = (idx0 + 1) % self.buffer_len;
        let frac = read_f - read_f.floor();
        buffer[idx0] * (1.0 - frac) + buffer[idx1] * frac
    }

    #[inline(always)]
    pub fn process_sample(&mut self, input: f32) -> f32 {
        // Smooth delay time interpolation to prevent tape zipper noise
        self.current_delay_samples += (self.target_delay_samples - self.current_delay_samples) * 0.005;

        // Read delayed signals
        let delayed_l = self.read_interpolated(&self.buffer_left, self.current_delay_samples);
        let delayed_r = self.read_interpolated(&self.buffer_right, self.current_delay_samples);

        // Feedback through lowpass damping
        let filtered_l = self.damping_filter_l.process_sample(delayed_l);
        let filtered_r = self.damping_filter_r.process_sample(delayed_r);

        let fb = self.params.feedback.clamp(0.0, 0.95);
        if self.params.ping_pong {
            // Cross-feedback ping-pong bounce
            self.buffer_left[self.write_pos] = input + filtered_r * fb;
            self.buffer_right[self.write_pos] = input + filtered_l * fb;
        } else {
            self.buffer_left[self.write_pos] = input + filtered_l * fb;
            self.buffer_right[self.write_pos] = input + filtered_r * fb;
        }

        self.write_pos = (self.write_pos + 1) % self.buffer_len;

        // Wet / Dry mix
        let wet_sample = (delayed_l + delayed_r) * 0.5;
        let wet = self.params.dry_wet.clamp(0.0, 1.0);
        input * (1.0 - wet) + wet_sample * wet
    }

    pub fn process_block(&mut self, buffer: &mut [f32]) {
        for s in buffer.iter_mut() {
            *s = self.process_sample(*s);
        }
    }
}
