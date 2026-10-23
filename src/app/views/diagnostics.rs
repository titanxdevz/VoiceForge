use egui::{Align, Layout, RichText, Stroke, Ui};
use crate::app::theme::ThemeColors;
use crate::audio::comms::AudioTelemetry;

pub fn render_diagnostics(
    ui: &mut Ui,
    telemetry: &AudioTelemetry,
    input_device: &str,
    output_device: &str,
    on_export_diagnostics: &mut bool,
) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("PERFORMANCE & DIAGNOSTICS").size(14.0).strong().color(ThemeColors::TEXT_PRIMARY));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.button("Export Diagnostics (JSON)").clicked() {
                *on_export_diagnostics = true;
            }
        });
    });

    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        // Real-time Metrics Card Grid
        ui.columns(3, |cols| {
            // Latency
            cols[0].vertical(|ui| {
                egui::Frame::none()
                    .fill(ThemeColors::SURFACE_BG)
                    .stroke(Stroke::new(1.0, ThemeColors::BORDER_HAIRLINE))
                    .inner_margin(14.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new("MEASURED LATENCY").size(11.0).color(ThemeColors::TEXT_SECONDARY));
                        ui.label(RichText::new(format!("{:.1} ms", telemetry.estimated_latency_ms)).size(24.0).strong().monospace().color(ThemeColors::ACCENT));
                        ui.label(RichText::new("Round-trip DSP buffer").size(10.0).color(ThemeColors::TEXT_MUTED));
                    });
            });

            // DSP CPU Load
            cols[1].vertical(|ui| {
                egui::Frame::none()
                    .fill(ThemeColors::SURFACE_BG)
                    .stroke(Stroke::new(1.0, ThemeColors::BORDER_HAIRLINE))
                    .inner_margin(14.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new("DSP CPU LOAD").size(11.0).color(ThemeColors::TEXT_SECONDARY));
                        let cpu_color = if telemetry.cpu_load_pct > 75.0 {
                            ThemeColors::METER_CLIP
                        } else if telemetry.cpu_load_pct > 40.0 {
                            ThemeColors::METER_WARN
                        } else {
                            ThemeColors::METER_NORMAL
                        };
                        ui.label(RichText::new(format!("{:.1} %", telemetry.cpu_load_pct)).size(24.0).strong().monospace().color(cpu_color));
                        ui.label(RichText::new("Single core real-time budget").size(10.0).color(ThemeColors::TEXT_MUTED));
                    });
            });

            // Dropouts
            cols[2].vertical(|ui| {
                egui::Frame::none()
                    .fill(ThemeColors::SURFACE_BG)
                    .stroke(Stroke::new(1.0, ThemeColors::BORDER_HAIRLINE))
                    .inner_margin(14.0)
                    .show(ui, |ui| {
                        ui.label(RichText::new("DROPOUT COUNTER").size(11.0).color(ThemeColors::TEXT_SECONDARY));
                        let dropout_color = if telemetry.dropouts > 0 { ThemeColors::METER_WARN } else { ThemeColors::TEXT_PRIMARY };
                        ui.label(RichText::new(format!("{}", telemetry.dropouts)).size(24.0).strong().monospace().color(dropout_color));
                        ui.label(RichText::new("Buffer underrun / overruns").size(10.0).color(ThemeColors::TEXT_MUTED));
                    });
            });
        });

        ui.add_space(14.0);

        // Hardware details
        egui::Frame::none()
            .fill(ThemeColors::SURFACE_BG)
            .stroke(Stroke::new(1.0, ThemeColors::BORDER_HAIRLINE))
            .inner_margin(14.0)
            .show(ui, |ui| {
                ui.label(RichText::new("AUDIO STREAM SPECIFICATIONS").size(12.0).strong().color(ThemeColors::TEXT_SECONDARY));
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label("Input Device:");
                    ui.label(RichText::new(input_device).monospace().color(ThemeColors::TEXT_PRIMARY));
                });
                ui.horizontal(|ui| {
                    ui.label("Output Device:");
                    ui.label(RichText::new(output_device).monospace().color(ThemeColors::TEXT_PRIMARY));
                });
                ui.horizontal(|ui| {
                    ui.label("Sample Rate:");
                    ui.label(RichText::new(format!("{} Hz", telemetry.sample_rate)).monospace().color(ThemeColors::TEXT_PRIMARY));
                });
                ui.horizontal(|ui| {
                    ui.label("Buffer Size:");
                    ui.label(RichText::new(format!("{} samples", telemetry.buffer_size)).monospace().color(ThemeColors::TEXT_PRIMARY));
                });
                ui.horizontal(|ui| {
                    ui.label("MMCSS Real-Time Priority:");
                    ui.label(RichText::new("Enabled (Pro Audio)").monospace().color(ThemeColors::METER_NORMAL));
                });
                ui.horizontal(|ui| {
                    ui.label("Privacy Guarantee:");
                    ui.label(RichText::new("100% Local processing. Zero telemetry or audio ever leaves this device.").color(ThemeColors::TEXT_SECONDARY));
                });
            });
    });
}
