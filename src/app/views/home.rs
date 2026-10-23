use egui::{Align, Align2, Color32, CornerRadius, FontId, Layout, Margin, Pos2, Rect, RichText, Sense, Stroke, Ui, Vec2};
use crate::app::theme::{ThemeColors, ThemeMetrics};
use crate::app::widgets::{render_circular_voice_tile, render_waveform_strip, SegmentedMeter};
use crate::presets::VoicePreset;

pub fn render_home(
    ui: &mut Ui,
    is_running: bool,
    active_preset_idx: usize,
    all_presets: &mut [VoicePreset],
    search_query: &mut String,
    category_filter: &mut String,
    on_toggle_engine: &mut bool,
    on_select_preset: &mut Option<String>,
    on_toggle_monitor: &mut bool,
    monitoring_enabled: bool,
    is_output_vbcable: bool,
    output_device_name: &str,
    on_fix_routing: &mut bool,
    in_db: f32,
    out_db: f32,
    in_peak: f32,
    out_peak: f32,
    in_clip_latched: &mut bool,
    out_clip_latched: &mut bool,
) {
    let active_id = if active_preset_idx < all_presets.len() {
        all_presets[active_preset_idx].id.clone()
    } else {
        String::new()
    };

    ui.spacing_mut().item_spacing = Vec2::new(ThemeMetrics::GRID, ThemeMetrics::GRID);

    // ==========================================
    // 1. TOP HEADER: ENGINE STATUS & ROUTING NOTICE
    // ==========================================
    ui.horizontal(|ui| {
        // Start / Stop Pill
        let pill_width = 132.0;
        let pill_height = 32.0;
        let (pill_rect, pill_resp) = ui.allocate_exact_size(Vec2::new(pill_width, pill_height), Sense::click());

        if pill_resp.clicked() {
            *on_toggle_engine = true;
        }

        if ui.is_rect_visible(pill_rect) {
            let painter = ui.painter();
            let pill_bg = if is_running { ThemeColors::SURFACE_ACTIVE } else { ThemeColors::SURFACE };
            painter.rect_filled(pill_rect, CornerRadius::same((pill_height * 0.5) as u8), pill_bg);
            let pill_border = if is_running { ThemeColors::ACCENT } else { ThemeColors::BORDER };
            painter.rect_stroke(pill_rect, CornerRadius::same((pill_height * 0.5) as u8), Stroke::new(1.0, pill_border), egui::StrokeKind::Inside);

            let time = ui.ctx().input(|i| i.time);
            let dot_x = pill_rect.min.x + 16.0;
            let dot_center = Pos2::new(dot_x, pill_rect.center().y);

            let (dot_color, dot_r) = if is_running {
                let pulse = (time * 4.0).sin().abs() as f32;
                (ThemeColors::SUCCESS, 4.5 + pulse * 1.5)
            } else {
                (ThemeColors::TEXT_FAINT, 4.0)
            };
            painter.circle_filled(dot_center, dot_r, dot_color);

            let text = if is_running { "STOP ENGINE" } else { "START ENGINE" };
            painter.text(
                Pos2::new(pill_rect.min.x + 30.0, pill_rect.center().y),
                Align2::LEFT_CENTER,
                text,
                FontId::proportional(11.0),
                ThemeColors::TEXT,
            );
        }

        ui.add_space(8.0);

        // Routing breadcrumb
        let routing_icon = if is_output_vbcable { "✓" } else { "⚠" };
        let routing_icon_color = if is_output_vbcable { ThemeColors::SUCCESS } else { ThemeColors::WARN };
        let routing_target = if is_output_vbcable { "CABLE Input" } else { output_device_name };

        ui.label(RichText::new(routing_icon).color(routing_icon_color).strong().size(12.0));
        let breadcrumb = format!("Mic → VoiceForge → {}", routing_target);
        ui.label(RichText::new(breadcrumb).font(FontId::proportional(11.0)).color(ThemeColors::TEXT_MUTED));

        // Right side: A/B Preview Toggle
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let mon_label = if monitoring_enabled { "Hear Myself: ON" } else { "Hear Myself: OFF" };
            let mon_bg = if monitoring_enabled { ThemeColors::SURFACE_ACTIVE } else { ThemeColors::SURFACE };
            let mon_stroke = if monitoring_enabled { Stroke::new(1.0, ThemeColors::ACCENT) } else { Stroke::new(1.0, ThemeColors::BORDER) };

            let btn = egui::Button::new(RichText::new(mon_label).font(FontId::proportional(11.0)).color(ThemeColors::TEXT))
                .fill(mon_bg)
                .stroke(mon_stroke)
                .corner_radius(CornerRadius::same(ThemeMetrics::RADIUS_CONTROL as u8));

            let btn_resp = ui.add(btn);
            if btn_resp.clicked() {
                *on_toggle_monitor = true;
            }
            btn_resp.on_hover_text("Route processed voice directly to your headphones to preview acoustics in real time.");
        });
    });

    if !is_output_vbcable {
        ui.add_space(2.0);
        egui::Frame::NONE
            .fill(Color32::from_rgb(32, 22, 14))
            .stroke(Stroke::new(1.0, ThemeColors::WARN))
            .corner_radius(CornerRadius::same(ThemeMetrics::RADIUS_CONTROL as u8))
            .inner_margin(6.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("⚠ Notice:").color(ThemeColors::WARN).strong().size(11.0));
                    ui.label(RichText::new(format!("Output set to '{}'. Virtual mic routing to Discord requires VB-CABLE.", output_device_name)).size(11.0).color(ThemeColors::TEXT));

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.add(egui::Button::new(RichText::new("Fix Routing →").size(10.0).strong().color(Color32::WHITE)).fill(ThemeColors::ACCENT_BLUE)).clicked() {
                            *on_fix_routing = true;
                        }
                    });
                });
            });
    }

    ui.add_space(4.0);

    // ==========================================
    // 2. MAIN VOICEMOD BODY (SPLIT: 70% VOICEBOX GRID | 30% INSPECTOR DRAWER)
    // ==========================================
    let total_width = ui.available_width();
    let inspector_width = 300.0;
    let grid_width = (total_width - inspector_width - ThemeMetrics::GRID).max(360.0);

    ui.horizontal_top(|ui| {
        // ------------------------------------------
        // LEFT: VOICEBOX AVATAR GRID
        // ------------------------------------------
        ui.allocate_ui_with_layout(Vec2::new(grid_width, ui.available_height()), Layout::top_down(Align::LEFT), |ui| {
            // Search and Category Bar (Voicemod style)
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(search_query)
                    .hint_text("🔍 Search voices...")
                    .desired_width(170.0)
                );

                ui.add_space(4.0);

                // Category Chips
                let categories = ["All", "Character", "Deep", "Dark", "Robot", "Cartoon", "Devices", "FX"];
                for cat in categories {
                    let is_sel = category_filter.as_str() == cat;
                    let bg = if is_sel { ThemeColors::ACCENT_SUBTLE } else { ThemeColors::SURFACE };
                    let stroke = if is_sel { Stroke::new(1.2, ThemeColors::ACCENT) } else { Stroke::new(1.0, ThemeColors::BORDER) };
                    let txt_col = if is_sel { ThemeColors::ACCENT } else { ThemeColors::TEXT_MUTED };

                    let chip = egui::Button::new(RichText::new(cat).font(FontId::proportional(11.0)).color(txt_col))
                        .fill(bg)
                        .stroke(stroke)
                        .corner_radius(CornerRadius::same(ThemeMetrics::RADIUS_PILL as u8));

                    if ui.add(chip).clicked() {
                        *category_filter = cat.to_string();
                    }
                }
            });

            ui.add_space(6.0);

            // Filter voices
            let query = search_query.to_lowercase();
            let cat = category_filter.to_lowercase();

            let filtered_indices: Vec<usize> = all_presets.iter().enumerate().filter_map(|(idx, p)| {
                let matches_query = query.is_empty() || p.name.to_lowercase().contains(&query) || p.sound_tag.to_lowercase().contains(&query);
                let matches_cat = cat == "all"
                    || (cat == "character" && (p.name.to_lowercase().contains("villain") || p.name.to_lowercase().contains("alien") || p.name.to_lowercase().contains("ghost") || p.name.to_lowercase().contains("anonymous")))
                    || (cat == "deep" && (p.name.to_lowercase().contains("deep") || p.name.to_lowercase().contains("giant")))
                    || (cat == "dark" && (p.name.to_lowercase().contains("dark") || p.name.to_lowercase().contains("demon") || p.name.to_lowercase().contains("villain")))
                    || (cat == "cartoon" && (p.name.to_lowercase().contains("cartoon") || p.name.to_lowercase().contains("chipmunk") || p.name.to_lowercase().contains("kid")))
                    || (cat == "robot" && (p.name.to_lowercase().contains("robot") || p.name.to_lowercase().contains("vocoder")))
                    || (cat == "devices" && (p.name.to_lowercase().contains("radio") || p.name.to_lowercase().contains("telephone") || p.name.to_lowercase().contains("megaphone") || p.name.to_lowercase().contains("walkie")))
                    || (cat == "fx" && (p.name.to_lowercase().contains("echo") || p.name.to_lowercase().contains("hall") || p.name.to_lowercase().contains("whisper") || p.name.to_lowercase().contains("reverb")));

                if matches_query && matches_cat {
                    Some(idx)
                } else {
                    None
                }
            }).collect();

            // Reflowing Circular Avatar Grid inside Scrollable Area
            let avail_w = ui.available_width();
            let item_w = 88.0;
            let num_cols = ((avail_w / (item_w + 4.0)).floor() as usize).max(3);

            egui::ScrollArea::vertical().max_height(ui.available_height() - 8.0).show(ui, |ui| {
                egui::Grid::new("voicemod_circular_grid")
                    .spacing(Vec2::new(4.0, 10.0))
                    .show(ui, |ui| {
                        for (i, &p_idx) in filtered_indices.iter().enumerate() {
                            let preset = &all_presets[p_idx];
                            let is_active = preset.id == active_id;

                            let resp = render_circular_voice_tile(
                                ui,
                                &preset.id,
                                &preset.name,
                                &preset.sound_tag,
                                preset.hotkey.as_deref(),
                                is_active,
                                item_w,
                            );

                            if resp.clicked() {
                                *on_select_preset = Some(preset.id.clone());
                            }

                            if (i + 1) % num_cols == 0 {
                                ui.end_row();
                            }
                        }
                    });
            });
        });

        ui.add_space(ThemeMetrics::GRID);

        // ------------------------------------------
        // RIGHT: VOICEMOD ACTIVE VOICE INSPECTOR DRAWER
        // ------------------------------------------
        ui.allocate_ui_with_layout(Vec2::new(inspector_width, ui.available_height()), Layout::top_down(Align::LEFT), |ui| {
            egui::Frame::NONE
                .fill(ThemeColors::SURFACE)
                .stroke(Stroke::new(1.0, ThemeColors::BORDER))
                .corner_radius(CornerRadius::same(ThemeMetrics::RADIUS_PANEL as u8))
                .inner_margin(Margin::same(12))
                .show(ui, |ui| {
                    if active_preset_idx < all_presets.len() {
                        let active_preset = &mut all_presets[active_preset_idx];

                        // Large Glowing Avatar Header (Image 1 Voicemod Style)
                        ui.vertical_centered(|ui| {
                            let big_avatar_r = 34.0;
                            let (badge_rect, _) = ui.allocate_exact_size(Vec2::splat(big_avatar_r * 2.0 + 8.0), Sense::hover());
                            let painter = ui.painter();
                            let center = badge_rect.center();

                            // Neon Cyan outer glow
                            painter.circle_stroke(center, big_avatar_r + 3.0, Stroke::new(2.5, ThemeColors::ACCENT));
                            painter.circle_filled(center, big_avatar_r, ThemeColors::SURFACE_ACTIVE);
                            
                            // Avatar vector art
                            crate::app::widgets::draw_procedural_avatar(painter, center, &active_preset.id, true);

                            ui.add_space(4.0);
                            ui.label(RichText::new(&active_preset.name).font(FontId::proportional(17.0)).strong().color(ThemeColors::TEXT));
                            ui.label(RichText::new(&active_preset.sound_tag).font(FontId::monospace(10.0)).color(ThemeColors::ACCENT_SOFT));

                            ui.add_space(4.0);
                            let hk_label = if let Some(ref hk) = active_preset.hotkey {
                                format!("Keybind: [{}]", hk)
                            } else {
                                "+ Add keybind".to_string()
                            };
                            let hk_btn = egui::Button::new(RichText::new(hk_label).font(FontId::monospace(10.0)).color(ThemeColors::TEXT_MUTED))
                                .fill(ThemeColors::SURFACE_RAISED)
                                .stroke(Stroke::new(1.0, ThemeColors::BORDER))
                                .corner_radius(CornerRadius::same(ThemeMetrics::RADIUS_SMALL as u8));
                            ui.add(hk_btn);
                        });

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(6.0);

                        // Voice Tuning Sliders with Cyan Fills (Image 1 Voicemod Style)
                        ui.label(RichText::new("VOICE PARAMETERS").font(FontId::monospace(10.0)).strong().color(ThemeColors::TEXT_MUTED));
                        ui.add_space(4.0);

                        let (mut pitch, mut formant) = active_preset.get_pitch_formant();
                        let orig_p = pitch;
                        let orig_f = formant;

                        // Pitch Slider (±12 semitones)
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Pitch").font(FontId::proportional(11.0)).color(ThemeColors::TEXT));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(RichText::new(format!("{:.1} st", pitch)).font(FontId::monospace(10.0)).color(ThemeColors::ACCENT));
                            });
                        });
                        ui.add(egui::Slider::new(&mut pitch, -12.0..=12.0).show_value(false).trailing_fill(true));

                        ui.add_space(4.0);

                        // Formant Shift Slider (±50%)
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Formant").font(FontId::proportional(11.0)).color(ThemeColors::TEXT));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(RichText::new(format!("{:.0}%", formant)).font(FontId::monospace(10.0)).color(ThemeColors::ACCENT));
                            });
                        });
                        ui.add(egui::Slider::new(&mut formant, -50.0..=50.0).show_value(false).trailing_fill(true));

                        if (pitch - orig_p).abs() > 0.05 || (formant - orig_f).abs() > 0.5 {
                            active_preset.set_pitch_formant(pitch, formant);
                        }

                        ui.add_space(4.0);

                        // Noise Separation Slider
                        let mut noise_val = active_preset.get_noise_suppression() * 100.0;
                        let orig_noise = noise_val;
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Noise Cut").font(FontId::proportional(11.0)).color(ThemeColors::TEXT));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(RichText::new(format!("{:.0}%", noise_val)).font(FontId::monospace(10.0)).color(ThemeColors::ACCENT));
                            });
                        });
                        ui.add(egui::Slider::new(&mut noise_val, 0.0..=100.0).show_value(false).trailing_fill(true));
                        if (noise_val - orig_noise).abs() > 0.5 {
                            active_preset.set_noise_suppression(noise_val * 0.01);
                        }

                        ui.add_space(4.0);

                        // Echo & Space Slider
                        let mut echo_val = active_preset.get_echo_mix() * 100.0;
                        let orig_echo = echo_val;
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Echo / Space").font(FontId::proportional(11.0)).color(ThemeColors::TEXT));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(RichText::new(format!("{:.0}%", echo_val)).font(FontId::monospace(10.0)).color(ThemeColors::ACCENT));
                            });
                        });
                        ui.add(egui::Slider::new(&mut echo_val, 0.0..=100.0).show_value(false).trailing_fill(true));
                        if (echo_val - orig_echo).abs() > 0.5 {
                            active_preset.set_echo_mix(echo_val * 0.01);
                        }

                        ui.add_space(4.0);

                        // Master Voice Volume Slider
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Voice Volume").font(FontId::proportional(11.0)).color(ThemeColors::TEXT));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(RichText::new(format!("{:.1} dB", active_preset.master_gain_db)).font(FontId::monospace(10.0)).color(ThemeColors::ACCENT));
                            });
                        });
                        ui.add(egui::Slider::new(&mut active_preset.master_gain_db, -18.0..=12.0).show_value(false).trailing_fill(true));

                        ui.add_space(8.0);

                        // Live Harmonic Waveform Strip
                        let strip_level = if is_running { out_db } else { -60.0 };
                        render_waveform_strip(ui, is_running, strip_level, ui.available_width(), 20.0);

                        ui.add_space(8.0);

                        // Segmented Level Meters
                        let display_in = if is_running { in_db } else { -60.0 };
                        let display_out = if is_running { out_db } else { -60.0 };
                        let display_in_peak = if is_running { in_peak } else { -60.0 };
                        let display_out_peak = if is_running { out_peak } else { -60.0 };

                        ui.horizontal(|ui| {
                            ui.label(RichText::new("IN").font(FontId::monospace(9.0)).color(ThemeColors::TEXT_MUTED));
                            ui.add(SegmentedMeter::new("meter_in", display_in, display_in_peak, in_clip_latched).size(ui.available_width() - 24.0, 12.0));
                        });
                        ui.add_space(2.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("OUT").font(FontId::monospace(9.0)).color(ThemeColors::TEXT_MUTED));
                            ui.add(SegmentedMeter::new("meter_out", display_out, display_out_peak, out_clip_latched).size(ui.available_width() - 24.0, 12.0));
                        });
                    }
                });
        });
    });
}
