use serde::{Deserialize, Serialize};
use std::time::Instant;

use crate::dsp::biquad::{BiquadFilter, BiquadParams};
use crate::dsp::delay::{EchoDelay, EchoDelayParams};
use crate::dsp::dynamics::{Compressor, CompressorParams, DeEsser, DeEsserParams, Limiter, LimiterParams, NoiseGate, NoiseGateParams};
use crate::dsp::modulation::{RingModParams, RingModulator, Vibrato, VibratoParams};
use crate::dsp::noise_suppression::{NoiseSuppression, NoiseSuppressionParams};
use crate::dsp::pitch_formant::{PitchFormantParams, PitchFormantShifter};
use crate::dsp::reverb::{FdnReverb, FdnReverbParams};
use crate::dsp::saturation::{Saturator, SaturatorParams};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EffectType {
    NoiseGate,
    NoiseSuppression,
    PitchFormant,
    Equalizer,
    Saturation,
    RingMod,
    Vibrato,
    EchoDelay,
    Compressor,
    DeEsser,
    Reverb,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffectParams {
    NoiseGate(NoiseGateParams),
    NoiseSuppression(NoiseSuppressionParams),
    PitchFormant(PitchFormantParams),
    Equalizer(BiquadParams),
    Saturation(SaturatorParams),
    RingMod(RingModParams),
    Vibrato(VibratoParams),
    EchoDelay(EchoDelayParams),
    Compressor(CompressorParams),
    DeEsser(DeEsserParams),
    Reverb(FdnReverbParams),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainEffectConfig {
    pub effect_type: EffectType,
    pub name: String,
    pub enabled: bool,
    pub wet_dry: f32, // 0.0 .. 1.0
    pub params: EffectParams,
}

pub enum EffectInstance {
    NoiseGate(NoiseGate),
    NoiseSuppression(NoiseSuppression),
    PitchFormant(PitchFormantShifter),
    Equalizer(BiquadFilter),
    Saturation(Saturator),
    RingMod(RingModulator),
    Vibrato(Vibrato),
    EchoDelay(EchoDelay),
    Compressor(Compressor),
    DeEsser(DeEsser),
    Reverb(FdnReverb),
}

pub struct ChainEffect {
    pub config: ChainEffectConfig,
    pub instance: EffectInstance,
    pub cpu_time_ns: u64,
}

pub struct EffectChain {
    pub effects: Vec<ChainEffect>,
    pub master_limiter: Limiter,
    pub sample_rate: f32,
    pub is_bypassed: bool,
    pub master_gain_linear: f32,
}

impl EffectChain {
    pub fn new(sample_rate: f32) -> Self {
        let sr = sample_rate.max(8000.0);
        let limiter = Limiter::new(sr, LimiterParams::default());
        Self {
            effects: Vec::new(),
            master_limiter: limiter,
            sample_rate: sr,
            is_bypassed: false,
            master_gain_linear: 1.0,
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate.max(8000.0);
        self.master_limiter.set_sample_rate(self.sample_rate);
        for effect in self.effects.iter_mut() {
            match &mut effect.instance {
                EffectInstance::NoiseGate(e) => e.set_sample_rate(self.sample_rate),
                EffectInstance::NoiseSuppression(e) => e.set_sample_rate(self.sample_rate),
                EffectInstance::PitchFormant(e) => e.set_sample_rate(self.sample_rate),
                EffectInstance::Equalizer(e) => e.set_sample_rate(self.sample_rate),
                EffectInstance::Saturation(_) => {},
                EffectInstance::RingMod(e) => e.set_sample_rate(self.sample_rate),
                EffectInstance::Vibrato(e) => e.set_sample_rate(self.sample_rate),
                EffectInstance::EchoDelay(e) => e.set_sample_rate(self.sample_rate),
                EffectInstance::Compressor(e) => e.set_sample_rate(self.sample_rate),
                EffectInstance::DeEsser(e) => e.set_sample_rate(self.sample_rate),
                EffectInstance::Reverb(e) => e.set_sample_rate(self.sample_rate),
            }
        }
    }

    pub fn set_master_gain_db(&mut self, gain_db: f32) {
        self.master_gain_linear = 10.0_f32.powf(gain_db / 20.0);
    }

    pub fn add_effect(&mut self, config: ChainEffectConfig) {
        let instance = match &config.params {
            EffectParams::NoiseGate(p) => EffectInstance::NoiseGate(NoiseGate::new(self.sample_rate, p.clone())),
            EffectParams::NoiseSuppression(p) => EffectInstance::NoiseSuppression(NoiseSuppression::new(self.sample_rate, p.clone())),
            EffectParams::PitchFormant(p) => EffectInstance::PitchFormant(PitchFormantShifter::new(self.sample_rate, p.clone())),
            EffectParams::Equalizer(p) => EffectInstance::Equalizer(BiquadFilter::new(self.sample_rate, p.clone())),
            EffectParams::Saturation(p) => EffectInstance::Saturation(Saturator::new(p.clone())),
            EffectParams::RingMod(p) => EffectInstance::RingMod(RingModulator::new(self.sample_rate, p.clone())),
            EffectParams::Vibrato(p) => EffectInstance::Vibrato(Vibrato::new(self.sample_rate, p.clone())),
            EffectParams::EchoDelay(p) => EffectInstance::EchoDelay(EchoDelay::new(self.sample_rate, p.clone())),
            EffectParams::Compressor(p) => EffectInstance::Compressor(Compressor::new(self.sample_rate, p.clone())),
            EffectParams::DeEsser(p) => EffectInstance::DeEsser(DeEsser::new(self.sample_rate, p.clone())),
            EffectParams::Reverb(p) => EffectInstance::Reverb(FdnReverb::new(self.sample_rate, p.clone())),
        };

        self.effects.push(ChainEffect {
            config,
            instance,
            cpu_time_ns: 0,
        });
    }

    pub fn remove_effect(&mut self, index: usize) {
        if index < self.effects.len() {
            self.effects.remove(index);
        }
    }

    pub fn reorder_effect(&mut self, from: usize, to: usize) {
        if from < self.effects.len() && to < self.effects.len() {
            let item = self.effects.remove(from);
            self.effects.insert(to, item);
        }
    }

    pub fn reset_all(&mut self) {
        for effect in self.effects.iter_mut() {
            match &mut effect.instance {
                EffectInstance::NoiseGate(e) => e.reset(),
                EffectInstance::NoiseSuppression(e) => e.reset(),
                EffectInstance::PitchFormant(e) => e.reset(),
                EffectInstance::Equalizer(e) => e.reset(),
                EffectInstance::Saturation(e) => e.reset(),
                EffectInstance::RingMod(e) => e.reset(),
                EffectInstance::Vibrato(e) => e.reset(),
                EffectInstance::EchoDelay(e) => e.reset(),
                EffectInstance::Compressor(e) => e.reset(),
                EffectInstance::DeEsser(e) => e.reset(),
                EffectInstance::Reverb(e) => e.reset(),
            }
        }
        self.master_limiter.reset();
    }

    /// Process a block of audio samples through the active effect chain.
    /// Master limiter is always running at the end to guarantee no clipping.
    pub fn process_block(&mut self, buffer: &mut [f32]) {
        if self.is_bypassed {
            // Even in bypass, apply master limiter for safety
            self.master_limiter.process_block(buffer);
            return;
        }

        // Apply each effect in the user-configured order
        for effect in self.effects.iter_mut() {
            if !effect.config.enabled {
                continue;
            }

            let start = Instant::now();
            match &mut effect.instance {
                EffectInstance::NoiseGate(e) => e.process_block(buffer),
                EffectInstance::NoiseSuppression(e) => e.process_block(buffer),
                EffectInstance::PitchFormant(e) => e.process_block(buffer),
                EffectInstance::Equalizer(e) => e.process_block(buffer),
                EffectInstance::Saturation(e) => e.process_block(buffer),
                EffectInstance::RingMod(e) => e.process_block(buffer),
                EffectInstance::Vibrato(e) => e.process_block(buffer),
                EffectInstance::EchoDelay(e) => e.process_block(buffer),
                EffectInstance::Compressor(e) => e.process_block(buffer),
                EffectInstance::DeEsser(e) => e.process_block(buffer),
                EffectInstance::Reverb(e) => e.process_block(buffer),
            }
            effect.cpu_time_ns = start.elapsed().as_nanos() as u64;
        }

        // Apply master gain
        if (self.master_gain_linear - 1.0).abs() > 1e-4 {
            for s in buffer.iter_mut() {
                *s *= self.master_gain_linear;
            }
        }

        // Always apply brickwall limiter at the end
        self.master_limiter.process_block(buffer);
    }
}
