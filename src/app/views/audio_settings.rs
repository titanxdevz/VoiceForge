use egui::{Align, Color32, Layout, RichText, Stroke, Ui};
use crate::app::theme::ThemeColors;
use crate::audio::device::{AudioDeviceInfo, DeviceManager, VbCableStatus};

pub fn render_audio_settings(
    ui: &mut Ui,
    input_devices: &[AudioDeviceInfo],
    output_devices: &[AudioDeviceInfo],
    current_input: &mut String,
    current_output: &mut String,
    current_buffer_size: &mut u32,
    cable_status: &VbCableStatus,
    start_minimized: &mut bool,
    start_with_windows: &mut bool,
    on_device_changed: &mut bool,
    on_autostart_toggled: &mut bool,
) {
    ui.label(RichText::new("AUDIO HARDWARE & ROUTING").size(14.0).strong().color(ThemeColors::TEXT_PRIMARY));
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        // Feedback Warning Alert if detected
        if cable_status.feedback_warning {
            egui::Frame::none()
                .fill(Color32::from_rgb(45, 20, 20))
                .stroke(Stroke::new(1.0, ThemeColors::METER_CLIP))
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.label(RichText::new("⚠ FEEDBACK LOOP DETECTED").strong().color(ThemeColors::METER_CLIP));
                    ui.label(RichText::new("Both Input and Output are set to VB-CABLE. This causes infinite screeching feedback!").size(12.0));
                    ui.label(RichText::new("Fix: Set Input to your physical Microphone (e.g. Realtek/USB Mic) and Output to 'CABLE Input'.").size(11.0).color(ThemeColors::TEXT_SECONDARY));
                });
            ui.add_space(8.0);
        }

        // VB-CABLE Status Card & Wizard
        egui::Frame::none()
            .fill(ThemeColors::SURFACE_BG)
            .stroke(Stroke::new(1.0, ThemeColors::BORDER_HAIRLINE))
            .inner_margin(14.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("VB-CABLE VIRTUAL AUDIO STATUS").size(12.0).strong().color(ThemeColors::TEXT_SECONDARY));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if cable_status.is_installed {
                            ui.label(RichText::new("✓ DETECTED").strong().color(ThemeColors::METER_NORMAL));
                        } else {
                            ui.label(RichText::new("NOT DETECTED").color(ThemeColors::METER_WARN));
                        }
                    });
                });

                ui.add_space(6.0);
                if cable_status.is_installed {
                    ui.label(RichText::new("VB-CABLE is installed. In Discord, Zoom, or games, select 'CABLE Output' as your Microphone.").size(12.0).color(ThemeColors::TEXT_SECONDARY));
                } else {
                    ui.label(RichText::new("VB-CABLE allows other apps to hear your changed voice. Download the free driver from vb-audio.com to stream into Discord/games.").size(12.0).color(ThemeColors::TEXT_SECONDARY));
                }
            });

        ui.add_space(14.0);

        // Hardware Device Pickers
        egui::Frame::none()
            .fill(ThemeColors::SURFACE_BG)
            .stroke(Stroke::new(1.0, ThemeColors::BORDER_HAIRLINE))
            .inner_margin(14.0)
            .show(ui, |ui| {
                ui.label(RichText::new("INPUT DEVICE (Physical Microphone)").size(12.0).strong().color(ThemeColors::TEXT_SECONDARY));
                ui.add_space(4.0);

                egui::ComboBox::from_id_salt("input_device_combo")
                    .selected_text(if current_input.is_empty() { "Default System Microphone" } else { current_input.as_str() })
                    .width(420.0)
                    .show_ui(ui, |ui| {
                        for dev in input_devices {
                            let label = if dev.is_default { format!("{} (Default)", dev.name) } else { dev.name.clone() };
                            if ui.selectable_value(current_input, dev.name.clone(), label).changed() {
                                *on_device_changed = true;
                            }
                        }
                    });

                ui.add_space(14.0);

                ui.label(RichText::new("OUTPUT DEVICE (Target / Virtual Mic)").size(12.0).strong().color(ThemeColors::TEXT_SECONDARY));
                ui.add_space(4.0);

                egui::ComboBox::from_id_salt("output_device_combo")
                    .selected_text(if current_output.is_empty() { "Default Audio Output" } else { current_output.as_str() })
                    .width(420.0)
                    .show_ui(ui, |ui| {
                        for dev in output_devices {
                            let mut label = dev.name.clone();
                            if dev.is_vbcable {
                                label = format!("{} ★ [Recommended for Discord]", dev.name);
                            } else if dev.is_default {
                                label = format!("{} (Default)", dev.name);
                            }
                            if ui.selectable_value(current_output, dev.name.clone(), label).changed() {
                                *on_device_changed = true;
                            }
                        }
                    });

                ui.add_space(14.0);

                // Buffer size / Latency selector
                ui.label(RichText::new("AUDIO BUFFER SIZE (Latency vs CPU)").size(12.0).strong().color(ThemeColors::TEXT_SECONDARY));
                ui.add_space(4.0);

                let buffer_options = [128, 256, 512, 1024];
                ui.horizontal(|ui| {
                    for &size in &buffer_options {
                        let text = match size {
                            128 => "128 (Ultra-Low ~2.7ms)",
                            256 => "256 (Balanced ~5.3ms)",
                            512 => "512 (Safe ~10.6ms)",
                            1024 => "1024 (High-Stability)",
                            _ => "",
                        };
                        if ui.selectable_value(current_buffer_size, size, text).changed() {
                            *on_device_changed = true;
                        }
                    }
                });
            });

        ui.add_space(14.0);

        // Background & Startup behavior
        egui::Frame::none()
            .fill(ThemeColors::SURFACE_BG)
            .stroke(Stroke::new(1.0, ThemeColors::BORDER_HAIRLINE))
            .inner_margin(14.0)
            .show(ui, |ui| {
                ui.label(RichText::new("SYSTEM & BACKGROUND SETTINGS").size(12.0).strong().color(ThemeColors::TEXT_SECONDARY));
                ui.add_space(6.0);

                if ui.checkbox(start_with_windows, "Start VoiceForge with Windows (Current User, No Admin required)").changed() {
                    *on_autostart_toggled = true;
                }

                ui.checkbox(start_minimized, "Start Minimized to System Tray");
                ui.label(RichText::new("Note: Closing the window minimizes VoiceForge to the tray to keep your voice engine running.").size(11.0).color(ThemeColors::TEXT_MUTED));
            });
    });
}
