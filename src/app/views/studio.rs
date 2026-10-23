use egui::{Align, Color32, Layout, RichText, Stroke, Ui, Vec2};
use crate::app::theme::ThemeColors;
use crate::app::widgets::RotaryKnob;
use crate::dsp::chain::{ChainEffectConfig, EffectParams, EffectType};
use crate::presets::VoicePreset;

pub fn render_studio(
    ui: &mut Ui,
    preset: &mut VoicePreset,
    on_chain_modified: &mut bool,
) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("EFFECT CHAIN").size(12.0).strong().color(ThemeColors::TEXT_SECONDARY));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new("Drag / Reorder / Bypass per effect").size(11.0).color(ThemeColors::TEXT_MUTED));
        });
    });

    ui.separator();

    let mut remove_idx = None;
    let mut move_up = None;
    let mut move_down = None;

    egui::ScrollArea::vertical().show(ui, |ui| {
        let effect_count = preset.effects.len();

        for (idx, effect) in preset.effects.iter_mut().enumerate() {
            let stroke = if effect.enabled {
                Stroke::new(1.0, ThemeColors::BORDER_HAIRLINE)
            } else {
                Stroke::new(1.0, ThemeColors::SURFACE_ACTIVE)
            };

            egui::Frame::none()
                .fill(ThemeColors::SURFACE_BG)
                .stroke(stroke)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    // Header row: enable checkbox, name, move up/down, remove
                    ui.horizontal(|ui| {
                        let mut enabled = effect.enabled;
                        if ui.checkbox(&mut enabled, "").changed() {
                            effect.enabled = enabled;
                            *on_chain_modified = true;
                        }

                        let title_color = if effect.enabled { ThemeColors::TEXT_PRIMARY } else { ThemeColors::TEXT_MUTED };
                        ui.label(RichText::new(&effect.name).strong().size(13.0).color(title_color));

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui.small_button("✕").clicked() {
                                remove_idx = Some(idx);
                            }
                            if idx < effect_count - 1 && ui.small_button("▼").clicked() {
                                move_down = Some(idx);
                            }
                            if idx > 0 && ui.small_button("▲").clicked() {
                                move_up = Some(idx);
                            }

                            ui.add(RotaryKnob::new("Mix", &mut effect.wet_dry, 0.0, 1.0, "").radius(14.0));
                        });
                    });

                    ui.add_space(8.0);

                    // Effect specific parameter editors
                    render_effect_params(ui, &mut effect.params, on_chain_modified);
                });

            ui.add_space(6.0);
        }

        ui.add_space(10.0);

        // Add Effect dropdown / bar
        ui.horizontal(|ui| {
            ui.label(RichText::new("+ Add Effect:").size(11.0).color(ThemeColors::TEXT_SECONDARY));

            if ui.button("+ Pitch/Formant").clicked() {
                preset.effects.push(ChainEffectConfig {
                    effect_type: EffectType::PitchFormant,
                    name: "Pitch & Formant".to_string(),
                    enabled: true,
                    wet_dry: 1.0,
                    params: EffectParams::PitchFormant(Default::default()),
                });
                *on_chain_modified = true;
            }
            if ui.button("+ Equalizer").clicked() {
                preset.effects.push(ChainEffectConfig {
                    effect_type: EffectType::Equalizer,
                    name: "Parametric EQ".to_string(),
                    enabled: true,
                    wet_dry: 1.0,
                    params: EffectParams::Equalizer(Default::default()),
                });
                *on_chain_modified = true;
            }
            if ui.button("+ Saturation").clicked() {
                preset.effects.push(ChainEffectConfig {
                    effect_type: EffectType::Saturation,
                    name: "Drive Saturation".to_string(),
                    enabled: true,
                    wet_dry: 0.5,
                    params: EffectParams::Saturation(Default::default()),
                });
                *on_chain_modified = true;
            }
            if ui.button("+ Reverb").clicked() {
                preset.effects.push(ChainEffectConfig {
                    effect_type: EffectType::Reverb,
                    name: "FDN Reverb".to_string(),
                    enabled: true,
                    wet_dry: 0.3,
                    params: EffectParams::Reverb(Default::default()),
                });
                *on_chain_modified = true;
            }
            if ui.button("+ Ring Mod").clicked() {
                preset.effects.push(ChainEffectConfig {
                    effect_type: EffectType::RingMod,
                    name: "Ring Modulator".to_string(),
                    enabled: true,
                    wet_dry: 0.8,
                    params: EffectParams::RingMod(Default::default()),
                });
                *on_chain_modified = true;
            }
            if ui.button("+ Compressor").clicked() {
                preset.effects.push(ChainEffectConfig {
                    effect_type: EffectType::Compressor,
                    name: "Dynamics Compressor".to_string(),
                    enabled: true,
                    wet_dry: 1.0,
                    params: EffectParams::Compressor(Default::default()),
                });
                *on_chain_modified = true;
            }
            if ui.button("+ Echo / Delay").clicked() {
                preset.effects.push(ChainEffectConfig {
                    effect_type: EffectType::EchoDelay,
                    name: "Echo Delay".to_string(),
                    enabled: true,
                    wet_dry: 0.4,
                    params: EffectParams::EchoDelay(Default::default()),
                });
                *on_chain_modified = true;
            }
            if ui.button("+ Noise Separation").clicked() {
                preset.effects.push(ChainEffectConfig {
                    effect_type: EffectType::NoiseSuppression,
                    name: "Noise Separation".to_string(),
                    enabled: true,
                    wet_dry: 1.0,
                    params: EffectParams::NoiseSuppression(Default::default()),
                });
                *on_chain_modified = true;
            }
        });
    });

    if let Some(idx) = remove_idx {
        preset.effects.remove(idx);
        *on_chain_modified = true;
    }
    if let Some(idx) = move_up {
        preset.effects.swap(idx, idx - 1);
        *on_chain_modified = true;
    }
    if let Some(idx) = move_down {
        preset.effects.swap(idx, idx + 1);
        *on_chain_modified = true;
    }
}

fn render_effect_params(ui: &mut Ui, params: &mut EffectParams, modified: &mut bool) {
    match params {
        EffectParams::PitchFormant(p) => {
            ui.horizontal(|ui| {
                if ui.add(RotaryKnob::new("Pitch", &mut p.pitch_semitones, -12.0, 12.0, "st").radius(18.0)).changed() {
                    *modified = true;
                }
                ui.add_space(16.0);
                if ui.add(RotaryKnob::new("Formant", &mut p.formant_shift_pct, -50.0, 50.0, "%").radius(18.0)).changed() {
                    *modified = true;
                }
            });
        }
        EffectParams::Equalizer(p) => {
            ui.horizontal(|ui| {
                if ui.add(RotaryKnob::new("Freq", &mut p.frequency, 40.0, 16000.0, "Hz").radius(18.0)).changed() {
                    *modified = true;
                }
                ui.add_space(16.0);
                if ui.add(RotaryKnob::new("Gain", &mut p.gain_db, -18.0, 18.0, "dB").radius(18.0)).changed() {
                    *modified = true;
                }
                ui.add_space(16.0);
                if ui.add(RotaryKnob::new("Q", &mut p.q, 0.2, 8.0, "").radius(18.0)).changed() {
                    *modified = true;
                }
            });
        }
        EffectParams::Saturation(p) => {
            ui.horizontal(|ui| {
                if ui.add(RotaryKnob::new("Drive", &mut p.drive_db, 0.0, 24.0, "dB").radius(18.0)).changed() {
                    *modified = true;
                }
                ui.add_space(16.0);
                if ui.add(RotaryKnob::new("Bias", &mut p.bias, -0.3, 0.3, "").radius(18.0)).changed() {
                    *modified = true;
                }
            });
        }
        EffectParams::Reverb(p) => {
            ui.horizontal(|ui| {
                if ui.add(RotaryKnob::new("Size", &mut p.room_size, 0.1, 0.98, "").radius(18.0)).changed() {
                    *modified = true;
                }
                ui.add_space(16.0);
                if ui.add(RotaryKnob::new("Damp", &mut p.damping, 0.0, 0.95, "").radius(18.0)).changed() {
                    *modified = true;
                }
                ui.add_space(16.0);
                if ui.add(RotaryKnob::new("Width", &mut p.width, 0.0, 1.0, "").radius(18.0)).changed() {
                    *modified = true;
                }
            });
        }
        EffectParams::RingMod(p) => {
            ui.horizontal(|ui| {
                if ui.add(RotaryKnob::new("Carrier", &mut p.carrier_freq_hz, 20.0, 1200.0, "Hz").radius(18.0)).changed() {
                    *modified = true;
                }
            });
        }
        EffectParams::Compressor(p) => {
            ui.horizontal(|ui| {
                if ui.add(RotaryKnob::new("Thresh", &mut p.threshold_db, -50.0, 0.0, "dB").radius(18.0)).changed() {
                    *modified = true;
                }
                ui.add_space(16.0);
                if ui.add(RotaryKnob::new("Ratio", &mut p.ratio, 1.0, 16.0, ":1").radius(18.0)).changed() {
                    *modified = true;
                }
                ui.add_space(16.0);
                if ui.add(RotaryKnob::new("Attack", &mut p.attack_ms, 0.5, 60.0, "ms").radius(18.0)).changed() {
                    *modified = true;
                }
                ui.add_space(16.0);
                if ui.add(RotaryKnob::new("Release", &mut p.release_ms, 10.0, 500.0, "ms").radius(18.0)).changed() {
                    *modified = true;
                }
            });
        }
        EffectParams::NoiseGate(p) => {
            ui.horizontal(|ui| {
                if ui.add(RotaryKnob::new("Thresh", &mut p.threshold_db, -70.0, -20.0, "dB").radius(18.0)).changed() {
                    *modified = true;
                }
                ui.add_space(16.0);
                if ui.add(RotaryKnob::new("Ratio", &mut p.ratio, 1.5, 16.0, ":1").radius(18.0)).changed() {
                    *modified = true;
                }
            });
        }
        EffectParams::DeEsser(p) => {
            ui.horizontal(|ui| {
                if ui.add(RotaryKnob::new("Freq", &mut p.frequency, 4000.0, 9000.0, "Hz").radius(18.0)).changed() {
                    *modified = true;
                }
                ui.add_space(16.0);
                if ui.add(RotaryKnob::new("Thresh", &mut p.threshold_db, -40.0, 0.0, "dB").radius(18.0)).changed() {
                    *modified = true;
                }
                ui.add_space(16.0);
                if ui.add(RotaryKnob::new("Reduce", &mut p.reduction_db, 0.0, 18.0, "dB").radius(18.0)).changed() {
                    *modified = true;
                }
            });
        }
        EffectParams::Vibrato(p) => {
            ui.horizontal(|ui| {
                if ui.add(RotaryKnob::new("Rate", &mut p.rate_hz, 0.5, 12.0, "Hz").radius(18.0)).changed() {
                    *modified = true;
                }
                ui.add_space(16.0);
                if ui.add(RotaryKnob::new("Depth", &mut p.depth_ms, 0.2, 8.0, "ms").radius(18.0)).changed() {
                    *modified = true;
                }
            });
        }
        EffectParams::EchoDelay(p) => {
            ui.horizontal(|ui| {
                if ui.add(RotaryKnob::new("Time", &mut p.delay_time_ms, 20.0, 1000.0, "ms").radius(18.0)).changed() {
                    *modified = true;
                }
                ui.add_space(16.0);
                if ui.add(RotaryKnob::new("Feedback", &mut p.feedback, 0.0, 0.95, "").radius(18.0)).changed() {
                    *modified = true;
                }
                ui.add_space(16.0);
                if ui.add(RotaryKnob::new("Damp", &mut p.damping_hz, 500.0, 12000.0, "Hz").radius(18.0)).changed() {
                    *modified = true;
                }
                ui.add_space(16.0);
                if ui.checkbox(&mut p.ping_pong, "Ping-Pong Stereo").changed() {
                    *modified = true;
                }
            });
        }
        EffectParams::NoiseSuppression(p) => {
            ui.horizontal(|ui| {
                if ui.add(RotaryKnob::new("Strength", &mut p.suppression_strength, 0.0, 1.0, "").radius(18.0)).changed() {
                    *modified = true;
                }
                ui.add_space(16.0);
                if ui.add(RotaryKnob::new("Sens", &mut p.threshold_db, -60.0, -20.0, "dB").radius(18.0)).changed() {
                    *modified = true;
                }
                ui.add_space(16.0);
                if ui.add(RotaryKnob::new("Release", &mut p.release_ms, 10.0, 300.0, "ms").radius(18.0)).changed() {
                    *modified = true;
                }
                ui.add_space(16.0);
                if ui.checkbox(&mut p.low_hum_cut, "AC Hum & Fan Cut").changed() {
                    *modified = true;
                }
            });
        }
    }
}
