use egui::{Align, Layout, RichText, Stroke, Ui};
use crate::app::theme::ThemeColors;
use crate::presets::VoicePreset;

pub fn render_presets_view(
    ui: &mut Ui,
    presets: &mut Vec<VoicePreset>,
    active_preset_id: &str,
    search_query: &mut String,
    on_select_preset: &mut Option<String>,
    on_export_preset: &mut Option<VoicePreset>,
    on_import_requested: &mut bool,
) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("VOICE LIBRARY").size(14.0).strong().color(ThemeColors::TEXT_PRIMARY));
        ui.add_space(16.0);
        ui.add(egui::TextEdit::singleline(search_query).hint_text("Search voices...").desired_width(220.0));

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.button("+ Import File").clicked() {
                *on_import_requested = true;
            }
        });
    });

    ui.separator();

    let query = search_query.to_lowercase();
    let mut duplicate_preset = None;

    egui::ScrollArea::vertical().show(ui, |ui| {
        for preset in presets.iter_mut() {
            if !query.is_empty() && !preset.name.to_lowercase().contains(&query) && !preset.sound_tag.to_lowercase().contains(&query) {
                continue;
            }

            let is_active = preset.id == active_preset_id;
            let stroke = if is_active {
                Stroke::new(1.5, ThemeColors::ACCENT)
            } else {
                Stroke::new(1.0, ThemeColors::BORDER_HAIRLINE)
            };

            egui::Frame::none()
                .fill(ThemeColors::SURFACE_BG)
                .stroke(stroke)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Favorite star toggle
                        let fav_icon = if preset.favorite { "★" } else { "☆" };
                        let fav_color = if preset.favorite { ThemeColors::METER_WARN } else { ThemeColors::TEXT_MUTED };
                        if ui.add(egui::Button::new(RichText::new(fav_icon).size(14.0).color(fav_color)).frame(false)).clicked() {
                            preset.favorite = !preset.favorite;
                        }

                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&preset.name).strong().size(14.0));
                                ui.label(RichText::new(&preset.sound_tag).size(11.0).monospace().color(ThemeColors::SECONDARY));
                            });
                            ui.label(RichText::new(&preset.description).size(11.0).color(ThemeColors::TEXT_SECONDARY));
                        });

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if !is_active {
                                if ui.button("Load Voice").clicked() {
                                    *on_select_preset = Some(preset.id.clone());
                                }
                            } else {
                                ui.label(RichText::new("CURRENT").size(11.0).strong().color(ThemeColors::ACCENT));
                            }

                            if ui.button("Export").clicked() {
                                *on_export_preset = Some(preset.clone());
                            }

                            if ui.button("Duplicate").clicked() {
                                let mut dup = preset.clone();
                                dup.id = format!("{}_copy", preset.id);
                                dup.name = format!("{} (Copy)", preset.name);
                                duplicate_preset = Some(dup);
                            }

                            if let Some(ref hk) = preset.hotkey {
                                ui.label(RichText::new(format!("[{}]", hk)).monospace().size(10.0).color(ThemeColors::TEXT_MUTED));
                            }
                        });
                    });
                });

            ui.add_space(6.0);
        }
    });

    if let Some(dup) = duplicate_preset {
        presets.push(dup);
    }
}
