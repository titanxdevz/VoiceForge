pub mod storage;

use serde::{Deserialize, Serialize};
use crate::dsp::chain::ChainEffectConfig;
use crate::dsp::biquad::{BiquadParams, FilterType};
use crate::dsp::delay::EchoDelayParams;
use crate::dsp::dynamics::{CompressorParams, DeEsserParams, NoiseGateParams};
use crate::dsp::modulation::{RingModParams, RingModWaveform, VibratoParams};
use crate::dsp::noise_suppression::NoiseSuppressionParams;
use crate::dsp::pitch_formant::PitchFormantParams;
use crate::dsp::reverb::FdnReverbParams;
use crate::dsp::saturation::{SaturationMode, SaturatorParams};
use crate::dsp::chain::{EffectParams, EffectType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoicePreset {
    pub id: String,
    pub name: String,
    pub description: String,
    pub sound_tag: String,
    pub favorite: bool,
    pub hotkey: Option<String>,
    pub master_gain_db: f32,
    pub effects: Vec<ChainEffectConfig>,
}

impl VoicePreset {
    pub fn get_pitch_formant(&self) -> (f32, f32) {
        for eff in &self.effects {
            if let EffectParams::PitchFormant(p) = &eff.params {
                return (p.pitch_semitones, p.formant_shift_pct);
            }
        }
        (0.0, 0.0)
    }

    pub fn set_pitch_formant(&mut self, pitch: f32, formant: f32) {
        for eff in &mut self.effects {
            if let EffectParams::PitchFormant(p) = &mut eff.params {
                p.pitch_semitones = pitch;
                p.formant_shift_pct = formant;
                return;
            }
        }
        // If not found, add one
        self.effects.push(ChainEffectConfig {
            effect_type: EffectType::PitchFormant,
            name: "Pitch & Formant".to_string(),
            enabled: true,
            wet_dry: 1.0,
            params: EffectParams::PitchFormant(PitchFormantParams {
                pitch_semitones: pitch,
                formant_shift_pct: formant,
                dry_wet: 1.0,
            }),
        });
    }

    pub fn get_noise_suppression(&self) -> f32 {
        for eff in &self.effects {
            if let EffectParams::NoiseSuppression(p) = &eff.params {
                if eff.enabled {
                    return p.suppression_strength;
                } else {
                    return 0.0;
                }
            }
        }
        0.0
    }

    pub fn set_noise_suppression(&mut self, strength: f32) {
        for eff in &mut self.effects {
            if let EffectParams::NoiseSuppression(p) = &mut eff.params {
                p.suppression_strength = strength;
                eff.enabled = strength > 0.01;
                return;
            }
        }
        if strength > 0.01 {
            self.effects.insert(0, ChainEffectConfig {
                effect_type: EffectType::NoiseSuppression,
                name: "Noise Separation".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::NoiseSuppression(NoiseSuppressionParams {
                    suppression_strength: strength,
                    ..Default::default()
                }),
            });
        }
    }

    pub fn get_echo_mix(&self) -> f32 {
        for eff in &self.effects {
            if let EffectParams::EchoDelay(p) = &eff.params {
                if eff.enabled {
                    return p.dry_wet;
                } else {
                    return 0.0;
                }
            }
        }
        0.0
    }

    pub fn set_echo_mix(&mut self, mix: f32) {
        for eff in &mut self.effects {
            if let EffectParams::EchoDelay(p) = &mut eff.params {
                p.dry_wet = mix;
                eff.wet_dry = mix;
                eff.enabled = mix > 0.01;
                return;
            }
        }
        if mix > 0.01 {
            self.effects.push(ChainEffectConfig {
                effect_type: EffectType::EchoDelay,
                name: "Echo Delay".to_string(),
                enabled: true,
                wet_dry: mix,
                params: EffectParams::EchoDelay(EchoDelayParams {
                    dry_wet: mix,
                    ..Default::default()
                }),
            });
        }
    }
}

pub fn get_all_builtin_presets() -> Vec<VoicePreset> {
    vec![
        create_deep_voice(),
        create_dark_voice(),
        create_demon(),
        create_villain(),
        create_cartoon(),
        create_chipmunk(),
        create_kid(),
        create_giant(),
        create_robot(),
        create_vocoder_robot(),
        create_radio(),
        create_telephone(),
        create_megaphone(),
        create_walkie_talkie(),
        create_alien(),
        create_ghost(),
        create_whisper(),
        create_echo(),
        create_hall(),
        create_cave(),
        create_stadium(),
        create_anonymous(),
        create_male_to_female(),
        create_natural(),
        create_custom(),
    ]
}

fn create_deep_voice() -> VoicePreset {
    VoicePreset {
        id: "deep_voice".to_string(),
        name: "Deep Voice".to_string(),
        description: "Lowers fundamental pitch and broadens throat resonance for authoritative depth.".to_string(),
        sound_tag: "Authoritative & Bass-heavy".to_string(),
        favorite: true,
        hotkey: Some("F1".to_string()),
        master_gain_db: 1.0,
        effects: vec![
            ChainEffectConfig {
                effect_type: EffectType::NoiseGate,
                name: "Gate".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::NoiseGate(NoiseGateParams::default()),
            },
            ChainEffectConfig {
                effect_type: EffectType::PitchFormant,
                name: "Pitch & Formant".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::PitchFormant(PitchFormantParams {
                    pitch_semitones: -4.5,
                    formant_shift_pct: -20.0,
                    dry_wet: 1.0,
                }),
            },
            ChainEffectConfig {
                effect_type: EffectType::Equalizer,
                name: "Warmth Boost".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::Equalizer(BiquadParams {
                    filter_type: FilterType::LowShelf,
                    frequency: 200.0,
                    q: 0.707,
                    gain_db: 3.5,
                }),
            },
            ChainEffectConfig {
                effect_type: EffectType::Compressor,
                name: "Compressor".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::Compressor(CompressorParams::default()),
            },
        ],
    }
}

fn create_dark_voice() -> VoicePreset {
    VoicePreset {
        id: "dark_voice".to_string(),
        name: "Dark Voice".to_string(),
        description: "Low pitch with downward formant shift, low-shelf boost and subtle analog saturation.".to_string(),
        sound_tag: "Ominous, Gritty & Heavy".to_string(),
        favorite: true,
        hotkey: Some("F2".to_string()),
        master_gain_db: 0.5,
        effects: vec![
            ChainEffectConfig {
                effect_type: EffectType::PitchFormant,
                name: "Pitch & Formant".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::PitchFormant(PitchFormantParams {
                    pitch_semitones: -6.0,
                    formant_shift_pct: -35.0,
                    dry_wet: 1.0,
                }),
            },
            ChainEffectConfig {
                effect_type: EffectType::Equalizer,
                name: "Low-Shelf EQ".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::Equalizer(BiquadParams {
                    filter_type: FilterType::LowShelf,
                    frequency: 180.0,
                    q: 0.8,
                    gain_db: 5.0,
                }),
            },
            ChainEffectConfig {
                effect_type: EffectType::Saturation,
                name: "Tube Saturation".to_string(),
                enabled: true,
                wet_dry: 0.45,
                params: EffectParams::Saturation(SaturatorParams {
                    drive_db: 4.5,
                    mode: SaturationMode::TubeWarmth,
                    bias: 0.08,
                    dry_wet: 0.45,
                }),
            },
        ],
    }
}

fn create_demon() -> VoicePreset {
    VoicePreset {
        id: "demon".to_string(),
        name: "Demon".to_string(),
        description: "Sub-harmonic guttural pitch drop, severe formant expansion and low-frequency ring modulation.".to_string(),
        sound_tag: "Guttural & Underworld".to_string(),
        favorite: false,
        hotkey: None,
        master_gain_db: 0.0,
        effects: vec![
            ChainEffectConfig {
                effect_type: EffectType::PitchFormant,
                name: "Pitch & Formant".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::PitchFormant(PitchFormantParams {
                    pitch_semitones: -9.0,
                    formant_shift_pct: -45.0,
                    dry_wet: 1.0,
                }),
            },
            ChainEffectConfig {
                effect_type: EffectType::RingMod,
                name: "Subharmonic Mod".to_string(),
                enabled: true,
                wet_dry: 0.35,
                params: EffectParams::RingMod(RingModParams {
                    carrier_freq_hz: 32.0,
                    waveform: RingModWaveform::Sine,
                    dry_wet: 0.35,
                }),
            },
            ChainEffectConfig {
                effect_type: EffectType::Saturation,
                name: "Drive".to_string(),
                enabled: true,
                wet_dry: 0.6,
                params: EffectParams::Saturation(SaturatorParams {
                    drive_db: 8.0,
                    mode: SaturationMode::AsymmetricDrive,
                    bias: 0.12,
                    dry_wet: 0.6,
                }),
            },
            ChainEffectConfig {
                effect_type: EffectType::Reverb,
                name: "Cave Reverb".to_string(),
                enabled: true,
                wet_dry: 0.3,
                params: EffectParams::Reverb(FdnReverbParams {
                    room_size: 0.85,
                    damping: 0.5,
                    width: 0.9,
                    dry_wet: 0.3,
                }),
            },
        ],
    }
}

fn create_villain() -> VoicePreset {
    VoicePreset {
        id: "villain".to_string(),
        name: "Villain".to_string(),
        description: "Sinister baritone pitch, close-mic compression and crisp top-end presence.".to_string(),
        sound_tag: "Sinister & Calculating".to_string(),
        favorite: true,
        hotkey: None,
        master_gain_db: 0.5,
        effects: vec![
            ChainEffectConfig {
                effect_type: EffectType::PitchFormant,
                name: "Pitch & Formant".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::PitchFormant(PitchFormantParams {
                    pitch_semitones: -3.5,
                    formant_shift_pct: -15.0,
                    dry_wet: 1.0,
                }),
            },
            ChainEffectConfig {
                effect_type: EffectType::Equalizer,
                name: "Presence Peak".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::Equalizer(BiquadParams {
                    filter_type: FilterType::PeakingEQ,
                    frequency: 3200.0,
                    q: 1.2,
                    gain_db: 3.5,
                }),
            },
            ChainEffectConfig {
                effect_type: EffectType::Compressor,
                name: "Tight Compressor".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::Compressor(CompressorParams {
                    threshold_db: -20.0,
                    ratio: 4.5,
                    attack_ms: 5.0,
                    release_ms: 80.0,
                    makeup_gain_db: 2.5,
                    knee_db: 3.0,
                }),
            },
        ],
    }
}

fn create_cartoon() -> VoicePreset {
    VoicePreset {
        id: "cartoon".to_string(),
        name: "Cartoon".to_string(),
        description: "Pitch up with elevated formant and light vibrato for playful animation voices.".to_string(),
        sound_tag: "Animated & Playful".to_string(),
        favorite: false,
        hotkey: None,
        master_gain_db: 0.0,
        effects: vec![
            ChainEffectConfig {
                effect_type: EffectType::PitchFormant,
                name: "Pitch & Formant".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::PitchFormant(PitchFormantParams {
                    pitch_semitones: 6.0,
                    formant_shift_pct: 30.0,
                    dry_wet: 1.0,
                }),
            },
            ChainEffectConfig {
                effect_type: EffectType::Vibrato,
                name: "Light Vibrato".to_string(),
                enabled: true,
                wet_dry: 0.4,
                params: EffectParams::Vibrato(VibratoParams {
                    rate_hz: 6.2,
                    depth_ms: 1.5,
                    dry_wet: 0.4,
                }),
            },
        ],
    }
}

fn create_chipmunk() -> VoicePreset {
    VoicePreset {
        id: "chipmunk".to_string(),
        name: "Chipmunk".to_string(),
        description: "Maximum octave-up pitch transposition and tiny vocal tract formant scaling.".to_string(),
        sound_tag: "High-Pitched & Squeaky".to_string(),
        favorite: true,
        hotkey: Some("F3".to_string()),
        master_gain_db: -1.0,
        effects: vec![
            ChainEffectConfig {
                effect_type: EffectType::PitchFormant,
                name: "High Pitch".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::PitchFormant(PitchFormantParams {
                    pitch_semitones: 10.5,
                    formant_shift_pct: 45.0,
                    dry_wet: 1.0,
                }),
            },
            ChainEffectConfig {
                effect_type: EffectType::DeEsser,
                name: "Tame Sibilance".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::DeEsser(DeEsserParams::default()),
            },
        ],
    }
}

fn create_kid() -> VoicePreset {
    VoicePreset {
        id: "kid".to_string(),
        name: "Kid".to_string(),
        description: "Natural child-like vocal tract scaling with gentle pitch lift.".to_string(),
        sound_tag: "Youthful & Bright".to_string(),
        favorite: false,
        hotkey: None,
        master_gain_db: 0.0,
        effects: vec![
            ChainEffectConfig {
                effect_type: EffectType::PitchFormant,
                name: "Child Scale".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::PitchFormant(PitchFormantParams {
                    pitch_semitones: 3.5,
                    formant_shift_pct: 22.0,
                    dry_wet: 1.0,
                }),
            },
        ],
    }
}

fn create_giant() -> VoicePreset {
    VoicePreset {
        id: "giant".to_string(),
        name: "Giant".to_string(),
        description: "Massive resonant chest cavity simulation with pitch step down.".to_string(),
        sound_tag: "Colossal & Resonant".to_string(),
        favorite: false,
        hotkey: None,
        master_gain_db: 0.0,
        effects: vec![
            ChainEffectConfig {
                effect_type: EffectType::PitchFormant,
                name: "Colossus Pitch".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::PitchFormant(PitchFormantParams {
                    pitch_semitones: -7.5,
                    formant_shift_pct: -45.0,
                    dry_wet: 1.0,
                }),
            },
            ChainEffectConfig {
                effect_type: EffectType::Reverb,
                name: "Body Resonance".to_string(),
                enabled: true,
                wet_dry: 0.25,
                params: EffectParams::Reverb(FdnReverbParams {
                    room_size: 0.6,
                    damping: 0.7,
                    width: 0.5,
                    dry_wet: 0.25,
                }),
            },
        ],
    }
}

fn create_robot() -> VoicePreset {
    VoicePreset {
        id: "robot".to_string(),
        name: "Robot".to_string(),
        description: "Metallic pulse modulation with robotic carrier pitch.".to_string(),
        sound_tag: "Robotic & Synthesized".to_string(),
        favorite: true,
        hotkey: Some("F4".to_string()),
        master_gain_db: 0.0,
        effects: vec![
            ChainEffectConfig {
                effect_type: EffectType::RingMod,
                name: "Robotizer".to_string(),
                enabled: true,
                wet_dry: 0.85,
                params: EffectParams::RingMod(RingModParams {
                    carrier_freq_hz: 65.0,
                    waveform: RingModWaveform::Square,
                    dry_wet: 0.85,
                }),
            },
        ],
    }
}

fn create_vocoder_robot() -> VoicePreset {
    VoicePreset {
        id: "vocoder_robot".to_string(),
        name: "Vocoder Robot".to_string(),
        description: "Harmonic pulse-train carrier modulation simulating classic electronic vocoders.".to_string(),
        sound_tag: "Electrified Dalek / Vocoder".to_string(),
        favorite: false,
        hotkey: None,
        master_gain_db: 0.0,
        effects: vec![
            ChainEffectConfig {
                effect_type: EffectType::RingMod,
                name: "Pulse Vocoder".to_string(),
                enabled: true,
                wet_dry: 0.9,
                params: EffectParams::RingMod(RingModParams {
                    carrier_freq_hz: 110.0,
                    waveform: RingModWaveform::PulseTrain,
                    dry_wet: 0.9,
                }),
            },
            ChainEffectConfig {
                effect_type: EffectType::Equalizer,
                name: "Bandpass Voice".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::Equalizer(BiquadParams {
                    filter_type: FilterType::BandPass,
                    frequency: 1500.0,
                    q: 1.5,
                    gain_db: 0.0,
                }),
            },
        ],
    }
}

fn create_radio() -> VoicePreset {
    VoicePreset {
        id: "radio".to_string(),
        name: "Radio".to_string(),
        description: "AM broadcast bandpass with vintage analog clipping.".to_string(),
        sound_tag: "Vintage AM Broadcast".to_string(),
        favorite: false,
        hotkey: None,
        master_gain_db: 0.0,
        effects: vec![
            ChainEffectConfig {
                effect_type: EffectType::Equalizer,
                name: "Low Cut".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::Equalizer(BiquadParams {
                    filter_type: FilterType::HighPass,
                    frequency: 450.0,
                    q: 0.707,
                    gain_db: 0.0,
                }),
            },
            ChainEffectConfig {
                effect_type: EffectType::Equalizer,
                name: "High Cut".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::Equalizer(BiquadParams {
                    filter_type: FilterType::LowPass,
                    frequency: 3800.0,
                    q: 0.707,
                    gain_db: 0.0,
                }),
            },
            ChainEffectConfig {
                effect_type: EffectType::Saturation,
                name: "Analog Saturation".to_string(),
                enabled: true,
                wet_dry: 0.5,
                params: EffectParams::Saturation(SaturatorParams {
                    drive_db: 5.0,
                    mode: SaturationMode::SoftClip,
                    bias: 0.0,
                    dry_wet: 0.5,
                }),
            },
        ],
    }
}

fn create_telephone() -> VoicePreset {
    VoicePreset {
        id: "telephone".to_string(),
        name: "Telephone".to_string(),
        description: "Classic 300Hz - 3400Hz narrow carbon mic telephone response.".to_string(),
        sound_tag: "Narrowband Landline".to_string(),
        favorite: false,
        hotkey: None,
        master_gain_db: 0.5,
        effects: vec![
            ChainEffectConfig {
                effect_type: EffectType::Equalizer,
                name: "Bandpass".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::Equalizer(BiquadParams {
                    filter_type: FilterType::BandPass,
                    frequency: 1200.0,
                    q: 2.2,
                    gain_db: 0.0,
                }),
            },
            ChainEffectConfig {
                effect_type: EffectType::Saturation,
                name: "Carbon Saturation".to_string(),
                enabled: true,
                wet_dry: 0.4,
                params: EffectParams::Saturation(SaturatorParams {
                    drive_db: 3.0,
                    mode: SaturationMode::TubeWarmth,
                    bias: 0.02,
                    dry_wet: 0.4,
                }),
            },
        ],
    }
}

fn create_megaphone() -> VoicePreset {
    VoicePreset {
        id: "megaphone".to_string(),
        name: "Megaphone".to_string(),
        description: "Harsh horn resonance with aggressive mid-range distortion.".to_string(),
        sound_tag: "Harsh Public Address Horn".to_string(),
        favorite: false,
        hotkey: None,
        master_gain_db: 0.0,
        effects: vec![
            ChainEffectConfig {
                effect_type: EffectType::Equalizer,
                name: "Horn Resonance".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::Equalizer(BiquadParams {
                    filter_type: FilterType::PeakingEQ,
                    frequency: 2100.0,
                    q: 3.5,
                    gain_db: 8.0,
                }),
            },
            ChainEffectConfig {
                effect_type: EffectType::Saturation,
                name: "Drive".to_string(),
                enabled: true,
                wet_dry: 0.7,
                params: EffectParams::Saturation(SaturatorParams {
                    drive_db: 10.0,
                    mode: SaturationMode::HardDistortion,
                    bias: 0.05,
                    dry_wet: 0.7,
                }),
            },
        ],
    }
}

fn create_walkie_talkie() -> VoicePreset {
    VoicePreset {
        id: "walkie_talkie".to_string(),
        name: "Walkie-Talkie".to_string(),
        description: "Tactical radio filter with background squelch gating.".to_string(),
        sound_tag: "Tactical VHF Radio".to_string(),
        favorite: false,
        hotkey: None,
        master_gain_db: 0.0,
        effects: vec![
            ChainEffectConfig {
                effect_type: EffectType::NoiseGate,
                name: "Radio Gate".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::NoiseGate(NoiseGateParams {
                    threshold_db: -40.0,
                    ratio: 16.0,
                    attack_ms: 1.0,
                    release_ms: 40.0,
                    hold_ms: 10.0,
                }),
            },
            ChainEffectConfig {
                effect_type: EffectType::Equalizer,
                name: "VHF Filter".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::Equalizer(BiquadParams {
                    filter_type: FilterType::BandPass,
                    frequency: 1800.0,
                    q: 1.8,
                    gain_db: 0.0,
                }),
            },
        ],
    }
}

fn create_alien() -> VoicePreset {
    VoicePreset {
        id: "alien".to_string(),
        name: "Alien".to_string(),
        description: "Dual harmonic frequency modulation with eerie pitch shifts.".to_string(),
        sound_tag: "Extraterrestrial Ring Mod".to_string(),
        favorite: false,
        hotkey: None,
        master_gain_db: 0.0,
        effects: vec![
            ChainEffectConfig {
                effect_type: EffectType::PitchFormant,
                name: "Alien Pitch".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::PitchFormant(PitchFormantParams {
                    pitch_semitones: 4.0,
                    formant_shift_pct: -25.0,
                    dry_wet: 1.0,
                }),
            },
            ChainEffectConfig {
                effect_type: EffectType::RingMod,
                name: "Alien Carrier".to_string(),
                enabled: true,
                wet_dry: 0.5,
                params: EffectParams::RingMod(RingModParams {
                    carrier_freq_hz: 320.0,
                    waveform: RingModWaveform::Sine,
                    dry_wet: 0.5,
                }),
            },
        ],
    }
}

fn create_ghost() -> VoicePreset {
    VoicePreset {
        id: "ghost".to_string(),
        name: "Ghost".to_string(),
        description: "Reverb washed spectral dissipation with whisper-ish noise modulation.".to_string(),
        sound_tag: "Ethereal & Haunting".to_string(),
        favorite: false,
        hotkey: None,
        master_gain_db: 0.0,
        effects: vec![
            ChainEffectConfig {
                effect_type: EffectType::PitchFormant,
                name: "Formant Shift".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::PitchFormant(PitchFormantParams {
                    pitch_semitones: 2.0,
                    formant_shift_pct: 15.0,
                    dry_wet: 1.0,
                }),
            },
            ChainEffectConfig {
                effect_type: EffectType::Reverb,
                name: "Ghostly Hall".to_string(),
                enabled: true,
                wet_dry: 0.65,
                params: EffectParams::Reverb(FdnReverbParams {
                    room_size: 0.95,
                    damping: 0.2,
                    width: 1.0,
                    dry_wet: 0.65,
                }),
            },
        ],
    }
}

fn create_whisper() -> VoicePreset {
    VoicePreset {
        id: "whisper".to_string(),
        name: "Whisper".to_string(),
        description: "Aggressive fundamental cut with high-frequency breath emphasis and gate.".to_string(),
        sound_tag: "Intimate & Breathy".to_string(),
        favorite: false,
        hotkey: None,
        master_gain_db: 1.0,
        effects: vec![
            ChainEffectConfig {
                effect_type: EffectType::Equalizer,
                name: "Remove Bass".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::Equalizer(BiquadParams {
                    filter_type: FilterType::HighPass,
                    frequency: 1000.0,
                    q: 0.707,
                    gain_db: 0.0,
                }),
            },
            ChainEffectConfig {
                effect_type: EffectType::Equalizer,
                name: "Air Boost".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::Equalizer(BiquadParams {
                    filter_type: FilterType::HighShelf,
                    frequency: 6000.0,
                    q: 0.707,
                    gain_db: 6.0,
                }),
            },
        ],
    }
}

fn create_echo() -> VoicePreset {
    VoicePreset {
        id: "echo".to_string(),
        name: "Echo".to_string(),
        description: "Rhythmic ping-pong stereo echo delay with high-frequency analog tape damping.".to_string(),
        sound_tag: "Rhythmic Echo Delay".to_string(),
        favorite: true,
        hotkey: None,
        master_gain_db: 0.0,
        effects: vec![
            ChainEffectConfig {
                effect_type: EffectType::NoiseSuppression,
                name: "Noise Separation".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::NoiseSuppression(NoiseSuppressionParams::default()),
            },
            ChainEffectConfig {
                effect_type: EffectType::EchoDelay,
                name: "Stereo Ping-Pong Delay".to_string(),
                enabled: true,
                wet_dry: 0.40,
                params: EffectParams::EchoDelay(EchoDelayParams {
                    delay_time_ms: 280.0,
                    feedback: 0.48,
                    damping_hz: 3800.0,
                    ping_pong: true,
                    dry_wet: 0.40,
                }),
            },
        ],
    }
}

fn create_hall() -> VoicePreset {
    VoicePreset {
        id: "hall".to_string(),
        name: "Hall".to_string(),
        description: "Lush concert hall acoustics with natural high-frequency roll-off.".to_string(),
        sound_tag: "Lush Concert Hall".to_string(),
        favorite: false,
        hotkey: None,
        master_gain_db: 0.0,
        effects: vec![
            ChainEffectConfig {
                effect_type: EffectType::Reverb,
                name: "Hall FDN".to_string(),
                enabled: true,
                wet_dry: 0.35,
                params: EffectParams::Reverb(FdnReverbParams {
                    room_size: 0.88,
                    damping: 0.35,
                    width: 0.9,
                    dry_wet: 0.35,
                }),
            },
        ],
    }
}

fn create_cave() -> VoicePreset {
    VoicePreset {
        id: "cave".to_string(),
        name: "Cave".to_string(),
        description: "Damp subterranean cavern reflections with long reverberant decay.".to_string(),
        sound_tag: "Cavernous & Resonant".to_string(),
        favorite: false,
        hotkey: None,
        master_gain_db: 0.0,
        effects: vec![
            ChainEffectConfig {
                effect_type: EffectType::Reverb,
                name: "Cavern Reverb".to_string(),
                enabled: true,
                wet_dry: 0.55,
                params: EffectParams::Reverb(FdnReverbParams {
                    room_size: 0.94,
                    damping: 0.6,
                    width: 1.0,
                    dry_wet: 0.55,
                }),
            },
        ],
    }
}

fn create_stadium() -> VoicePreset {
    VoicePreset {
        id: "stadium".to_string(),
        name: "Stadium".to_string(),
        description: "Expansive outdoor stadium reverberation with open stereo width.".to_string(),
        sound_tag: "Arena / Stadium PA".to_string(),
        favorite: false,
        hotkey: None,
        master_gain_db: 0.0,
        effects: vec![
            ChainEffectConfig {
                effect_type: EffectType::Reverb,
                name: "Stadium Space".to_string(),
                enabled: true,
                wet_dry: 0.5,
                params: EffectParams::Reverb(FdnReverbParams {
                    room_size: 0.92,
                    damping: 0.25,
                    width: 1.0,
                    dry_wet: 0.5,
                }),
            },
        ],
    }
}

fn create_anonymous() -> VoicePreset {
    VoicePreset {
        id: "anonymous".to_string(),
        name: "Anonymous".to_string(),
        description: "Identity disguise combining downward pitch, shifted formants and vocal masking.".to_string(),
        sound_tag: "Disguised & Anonymous".to_string(),
        favorite: true,
        hotkey: Some("F5".to_string()),
        master_gain_db: 0.0,
        effects: vec![
            ChainEffectConfig {
                effect_type: EffectType::PitchFormant,
                name: "Voice Disguise".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::PitchFormant(PitchFormantParams {
                    pitch_semitones: -3.8,
                    formant_shift_pct: 12.0, // Disconnect pitch and formant for identity disruption
                    dry_wet: 1.0,
                }),
            },
            ChainEffectConfig {
                effect_type: EffectType::Equalizer,
                name: "Band Filter".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::Equalizer(BiquadParams {
                    filter_type: FilterType::BandPass,
                    frequency: 1400.0,
                    q: 1.4,
                    gain_db: 0.0,
                }),
            },
        ],
    }
}

fn create_male_to_female() -> VoicePreset {
    VoicePreset {
        id: "male_to_female".to_string(),
        name: "Male↔Female Shift".to_string(),
        description: "Bi-directional gender formant transformation with pitch realignment.".to_string(),
        sound_tag: "Feminine / Masculine Shift".to_string(),
        favorite: true,
        hotkey: None,
        master_gain_db: 0.0,
        effects: vec![
            ChainEffectConfig {
                effect_type: EffectType::PitchFormant,
                name: "Gender Shifter".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::PitchFormant(PitchFormantParams {
                    pitch_semitones: 4.2,
                    formant_shift_pct: 18.0,
                    dry_wet: 1.0,
                }),
            },
            ChainEffectConfig {
                effect_type: EffectType::DeEsser,
                name: "DeEsser".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::DeEsser(DeEsserParams::default()),
            },
        ],
    }
}

fn create_natural() -> VoicePreset {
    VoicePreset {
        id: "natural".to_string(),
        name: "Natural Clean".to_string(),
        description: "Studio broadcast clarity: AI/spectral noise separation, low-hum cut, de-esser and transparent compression.".to_string(),
        sound_tag: "Clean Broadcast Studio".to_string(),
        favorite: true,
        hotkey: Some("F6".to_string()),
        master_gain_db: 1.0,
        effects: vec![
            ChainEffectConfig {
                effect_type: EffectType::NoiseSuppression,
                name: "Noise Separation".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::NoiseSuppression(NoiseSuppressionParams {
                    suppression_strength: 0.70,
                    threshold_db: -42.0,
                    low_hum_cut: true,
                    release_ms: 50.0,
                }),
            },
            ChainEffectConfig {
                effect_type: EffectType::NoiseGate,
                name: "Studio Gate".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::NoiseGate(NoiseGateParams {
                    threshold_db: -48.0,
                    ratio: 4.0,
                    attack_ms: 2.0,
                    release_ms: 120.0,
                    hold_ms: 30.0,
                }),
            },
            ChainEffectConfig {
                effect_type: EffectType::Equalizer,
                name: "Low Cut".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::Equalizer(BiquadParams {
                    filter_type: FilterType::HighPass,
                    frequency: 80.0,
                    q: 0.707,
                    gain_db: 0.0,
                }),
            },
            ChainEffectConfig {
                effect_type: EffectType::Compressor,
                name: "Vocal Leveler".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::Compressor(CompressorParams {
                    threshold_db: -18.0,
                    ratio: 2.5,
                    attack_ms: 15.0,
                    release_ms: 140.0,
                    makeup_gain_db: 2.0,
                    knee_db: 6.0,
                }),
            },
            ChainEffectConfig {
                effect_type: EffectType::DeEsser,
                name: "De-Esser".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::DeEsser(DeEsserParams::default()),
            },
        ],
    }
}

fn create_custom() -> VoicePreset {
    VoicePreset {
        id: "custom".to_string(),
        name: "Custom".to_string(),
        description: "User scratchpad for custom sound design experiments and effect chaining.".to_string(),
        sound_tag: "Configurable User Chain".to_string(),
        favorite: false,
        hotkey: None,
        master_gain_db: 0.0,
        effects: vec![
            ChainEffectConfig {
                effect_type: EffectType::PitchFormant,
                name: "Pitch & Formant".to_string(),
                enabled: true,
                wet_dry: 1.0,
                params: EffectParams::PitchFormant(PitchFormantParams::default()),
            },
        ],
    }
}
