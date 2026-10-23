pub mod theme;
pub mod widgets;
pub mod views;

use std::path::PathBuf;
use egui::{Align, CentralPanel, Color32, CornerRadius, FontId, Layout, RichText, SidePanel, Stroke, TopBottomPanel, Vec2};
use crate::app::theme::{ThemeColors, ThemeMetrics};
use crate::app::widgets::{render_hud_toggle, render_nav_item, render_titlebar};
use crate::app::views::audio_settings::render_audio_settings;
use crate::app::views::diagnostics::render_diagnostics;
use crate::app::views::home::render_home;
use crate::app::views::presets_view::render_presets_view;
use crate::app::views::studio::render_studio;
use crate::audio::comms::{AudioCommand, AudioTelemetry};
use crate::audio::AudioEngine;
use crate::presets::storage::PresetStorage;
use crate::presets::{get_all_builtin_presets, VoicePreset};
use crate::system::AutostartManager;
use crate::tray::SystemTrayManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Home,
    VoiceStudio,
    Presets,
    AudioSettings,
    Diagnostics,
}

pub struct VoiceForgeApp {
    active_tab: ActiveTab,
    presets: Vec<VoicePreset>,
    active_preset_id: String,
    search_query: String,
    category_filter: String,

    // Audio Engine & Global Voicemod Controls
    audio_engine: AudioEngine,
    telemetry: AudioTelemetry,
    is_muted: bool,
    monitoring_enabled: bool,
    voice_changer_enabled: bool,
    noise_reduction_pct: f32, // 0.0 .. 100.0%

    // Settings
    start_minimized: bool,
    start_with_windows: bool,
    reduce_motion: bool,
    is_maximized: bool,
    preset_storage: PresetStorage,
    tray_manager: Option<SystemTrayManager>,

    // Meters & clipping
    in_peak_hold: f32,
    out_peak_hold: f32,
    in_clip_latched: bool,
    out_clip_latched: bool,
}

impl VoiceForgeApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply_theme(&cc.egui_ctx);

        let storage = PresetStorage::new();
        let mut presets = get_all_builtin_presets();
        let user_presets = storage.load_all_user_presets();
        for up in user_presets {
            if !presets.iter().any(|p| p.id == up.id) {
                presets.push(up);
            }
        }

        let active_id = presets.first().map(|p| p.id.clone()).unwrap_or_else(|| "deep_voice".to_string());
        let engine = AudioEngine::new().expect("Failed to initialize audio subsystem");
        let tray = SystemTrayManager::new().ok();

        Self {
            active_tab: ActiveTab::Home,
            presets,
            active_preset_id: active_id,
            search_query: String::new(),
            category_filter: "All".to_string(),
            audio_engine: engine,
            telemetry: AudioTelemetry::default(),
            is_muted: false,
            monitoring_enabled: false,
            voice_changer_enabled: true,
            noise_reduction_pct: 65.0,
            start_minimized: false,
            start_with_windows: false,
            reduce_motion: false,
            is_maximized: false,
            preset_storage: storage,
            tray_manager: tray,
            in_peak_hold: -60.0,
            out_peak_hold: -60.0,
            in_clip_latched: false,
            out_clip_latched: false,
        }
    }

    fn active_preset_mut(&mut self) -> &mut VoicePreset {
        let id = self.active_preset_id.clone();
        if let Some(pos) = self.presets.iter().position(|p| p.id == id) {
            &mut self.presets[pos]
        } else {
            &mut self.presets[0]
        }
    }

    fn active_preset(&self) -> &VoicePreset {
        let id = &self.active_preset_id;
        self.presets.iter().find(|p| &p.id == id).unwrap_or(&self.presets[0])
    }

    fn select_preset(&mut self, id: String) {
        if let Some(p) = self.presets.iter().find(|p| p.id == id).cloned() {
            self.active_preset_id = id;
            if self.audio_engine.is_running() {
                self.audio_engine.send_command(AudioCommand::SetPreset(p));
            }
        }
    }

    fn toggle_engine(&mut self) {
        if self.audio_engine.is_running() {
            self.audio_engine.stop();
            // Reset meters to floor
            self.telemetry.input_peak_db = -60.0;
            self.telemetry.output_peak_db = -60.0;
            self.in_peak_hold = -60.0;
            self.out_peak_hold = -60.0;
        } else {
            self.audio_engine.monitoring_enabled = self.monitoring_enabled;
            self.audio_engine.monitoring_active.store(self.monitoring_enabled, std::sync::atomic::Ordering::Relaxed);
            let preset = self.active_preset().clone();
            if let Err(e) = self.audio_engine.start(preset) {
                eprintln!("Failed to start audio engine: {}", e);
            }
        }

        if let Some(ref tray) = self.tray_manager {
            tray.update_state(self.audio_engine.is_running(), self.is_muted);
        }
    }
}

impl eframe::App for VoiceForgeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll telemetry from Audio Engine
        if let Some(t) = self.audio_engine.try_receive_telemetry() {
            self.telemetry = t;
            // Ballistic peak hold decay
            self.in_peak_hold = (self.in_peak_hold - 0.4).max(t.input_peak_db);
            self.out_peak_hold = (self.out_peak_hold - 0.4).max(t.output_peak_db);
        }

        let mut win_min = false;
        let mut win_max = false;
        let mut win_close = false;

        // 1. Custom Frameless Titlebar
        TopBottomPanel::top("custom_titlebar")
            .exact_height(ThemeMetrics::TITLEBAR_HEIGHT)
            .show(ctx, |ui| {
                render_titlebar(
                    ui,
                    "VoiceForge • Studio Edition",
                    &mut win_min,
                    &mut win_max,
                    &mut win_close,
                );
            });

        if win_min {
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        }
        if win_max {
            self.is_maximized = !self.is_maximized;
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(self.is_maximized));
        }
        if win_close {
            // Minimize to system tray so audio engine keeps streaming
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }

        // 2. 200px Left Sidebar with 16px Vector Icons
        SidePanel::left("nav_panel")
            .resizable(false)
            .exact_width(ThemeMetrics::SIDEBAR_WIDTH)
            .show(ctx, |ui| {
                ui.add_space(12.0);

                let tabs = [
                    (ActiveTab::Home, "Dashboard", 0),
                    (ActiveTab::VoiceStudio, "Voice Studio", 1),
                    (ActiveTab::Presets, "Voice Library", 2),
                    (ActiveTab::AudioSettings, "Audio & Devices", 3),
                    (ActiveTab::Diagnostics, "Diagnostics", 4),
                ];

                for (tab, label, icon_id) in tabs {
                    let is_active = self.active_tab == tab;
                    if render_nav_item(ui, label, is_active, icon_id).clicked() {
                        self.active_tab = tab;
                    }
                    ui.add_space(3.0);
                }

                // Bottom version info
                ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                    ui.add_space(8.0);
                    ui.label(RichText::new("v1.0").font(FontId::monospace(9.0)).color(ThemeColors::TEXT_FAINT));
                });
            });

        // 3. Persistent Voicemod-Style Master Audio Dock
        TopBottomPanel::bottom("voicemod_master_dock")
            .exact_height(ThemeMetrics::MASTER_DOCK_HEIGHT)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add_space(8.0);

                    // 1. Voice Changer ON/OFF Toggle
                    let vc_resp = render_hud_toggle(
                        ui,
                        "hud_voice_changer",
                        "Voice Changer",
                        self.voice_changer_enabled,
                        ThemeColors::ACCENT,
                        0,
                    );
                    if vc_resp.clicked() {
                        self.voice_changer_enabled = !self.voice_changer_enabled;
                        self.audio_engine.send_command(AudioCommand::SetBypass(!self.voice_changer_enabled));
                    }
                    vc_resp.on_hover_text("When OFF, your natural voice passes through cleanly with zero effects.");

                    ui.add_space(4.0);

                    // 2. Hear Myself (Monitoring) Toggle with Headphones Warning
                    let mon_resp = render_hud_toggle(
                        ui,
                        "hud_hear_myself",
                        "Hear Myself",
                        self.monitoring_enabled,
                        ThemeColors::SUCCESS,
                        1,
                    );
                    if mon_resp.clicked() {
                        self.monitoring_enabled = !self.monitoring_enabled;
                        self.audio_engine.set_monitoring(self.monitoring_enabled);
                    }
                    mon_resp.on_hover_text("Real-time voice monitoring.\n⚠ Wear headphones to prevent acoustic feedback!");

                    ui.add_space(4.0);

                    // 3. Mute Mic Toggle
                    let mute_resp = render_hud_toggle(
                        ui,
                        "hud_mute_mic",
                        if self.is_muted { "Muted" } else { "Mute Mic" },
                        !self.is_muted,
                        ThemeColors::TEXT,
                        2,
                    );
                    if mute_resp.clicked() {
                        self.is_muted = !self.is_muted;
                        let gain = if self.is_muted { -120.0 } else { self.active_preset().master_gain_db };
                        self.audio_engine.send_command(AudioCommand::SetMasterGain(gain));
                        if let Some(ref tray) = self.tray_manager {
                            tray.update_state(self.audio_engine.is_running(), self.is_muted);
                        }
                    }
                    mute_resp.on_hover_text("Instantly silences output to Discord, games, and streams.");

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // 4. Background Noise Separation Slider
                    ui.label(RichText::new("Noise Cut:").font(FontId::proportional(11.0)).color(ThemeColors::TEXT_MUTED));
                    let slider = egui::Slider::new(&mut self.noise_reduction_pct, 0.0..=100.0)
                        .show_value(false)
                        .trailing_fill(true);
                    let s_resp = ui.add_sized([90.0, 18.0], slider);
                    if s_resp.changed() {
                        let strength = self.noise_reduction_pct * 0.01;
                        let active = self.active_preset_mut();
                        for eff in &mut active.effects {
                            if let crate::dsp::chain::EffectParams::NoiseSuppression(ns) = &mut eff.params {
                                ns.suppression_strength = strength;
                            }
                        }
                        let preset_clone = active.clone();
                        if self.audio_engine.is_running() {
                            self.audio_engine.send_command(AudioCommand::SetPreset(preset_clone));
                        }
                    }
                    ui.label(RichText::new(format!("{:.0}%", self.noise_reduction_pct)).font(FontId::monospace(11.0)).color(ThemeColors::TEXT));
                    s_resp.on_hover_text("Adaptive Spectral Noise Separation: Strips background fan hum, air conditioning, and room hiss.");

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.add_space(8.0);

                        // 5. Clean Mic Panic / Reset Button
                        let clean_btn = egui::Button::new(RichText::new("✨ Clean Mic").font(FontId::proportional(11.0)).strong().color(Color32::WHITE))
                            .fill(ThemeColors::SURFACE_ACTIVE)
                            .stroke(Stroke::new(1.0, ThemeColors::ACCENT_SOFT))
                            .corner_radius(CornerRadius::same(ThemeMetrics::RADIUS_SMALL as u8));
                        if ui.add(clean_btn).on_hover_text("Panic shortcut: Instantly loads the Natural Broadcast studio voice!").clicked() {
                            self.select_preset("natural".to_string());
                        }
                    });
                });
            });

        // 4. Persistent Bottom Status Bar with Truncated Device Names & Tooltips
        TopBottomPanel::bottom("status_bar")
            .exact_height(ThemeMetrics::STATUSBAR_HEIGHT)
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    let in_name = if self.audio_engine.current_input_device.is_empty() { "Default Mic" } else { self.audio_engine.current_input_device.as_str() };
                    let out_name = if self.audio_engine.current_output_device.is_empty() { "Default Out" } else { self.audio_engine.current_output_device.as_str() };

                    let trunc_in = if in_name.len() > 22 { format!("{}...", &in_name[..20]) } else { in_name.to_string() };
                    let trunc_out = if out_name.len() > 22 { format!("{}...", &out_name[..20]) } else { out_name.to_string() };

                    let in_label = ui.label(RichText::new(format!("IN: {}", trunc_in)).font(FontId::monospace(10.0)).color(ThemeColors::TEXT_MUTED));
                    in_label.on_hover_text(format!("Input Device: {}", in_name));

                    ui.separator();

                    let out_label = ui.label(RichText::new(format!("OUT: {}", trunc_out)).font(FontId::monospace(10.0)).color(ThemeColors::TEXT_MUTED));
                    out_label.on_hover_text(format!("Output Device: {}", out_name));

                    ui.separator();
                    ui.label(RichText::new(format!("{} Hz • {} spl", self.audio_engine.current_sample_rate, self.audio_engine.current_buffer_size)).font(FontId::monospace(10.0)).color(ThemeColors::TEXT_FAINT));

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let latency_text = format!("CPU: {:.1}% | DSP: {:.1}ms", self.telemetry.cpu_load_pct, self.telemetry.estimated_latency_ms);
                        ui.label(RichText::new(latency_text).font(FontId::monospace(10.0)).color(ThemeColors::TEXT_MUTED));
                    });
                });
            });

        // 4. Main View Content Panel
        CentralPanel::default().show(ctx, |ui| {
            ui.add_space(4.0);

            let mut toggle_engine = false;
            let mut select_preset = None;
            let mut toggle_monitor = false;
            let mut chain_modified = false;
            let mut export_preset = None;
            let mut import_requested = false;
            let mut device_changed = false;
            let mut autostart_toggled = false;
            let mut export_diagnostics = false;
            let mut fix_routing = false;

            let cable_stat = self.audio_engine.device_manager().detect_vbcable(
                &self.audio_engine.current_input_device,
                &self.audio_engine.current_output_device,
            );
            let is_output_vbcable = cable_stat.output_device_name.as_deref() == Some(&self.audio_engine.current_output_device)
                || self.audio_engine.current_output_device.to_lowercase().contains("cable");

            match self.active_tab {
                ActiveTab::Home => {
                    let in_db = self.telemetry.input_peak_db;
                    let out_db = self.telemetry.output_peak_db;
                    let in_peak = self.in_peak_hold;
                    let out_peak = self.out_peak_hold;
                    let mon = self.monitoring_enabled;
                    let out_dev = self.audio_engine.current_output_device.clone();
                    let is_running = self.audio_engine.is_running();
                    let active_id = self.active_preset_id.clone();
                    let active_idx = self.presets.iter().position(|p| p.id == active_id).unwrap_or(0);

                    render_home(
                        ui,
                        is_running,
                        active_idx,
                        &mut self.presets,
                        &mut self.search_query,
                        &mut self.category_filter,
                        &mut toggle_engine,
                        &mut select_preset,
                        &mut toggle_monitor,
                        mon,
                        is_output_vbcable,
                        &out_dev,
                        &mut fix_routing,
                        in_db,
                        out_db,
                        in_peak,
                        out_peak,
                        &mut self.in_clip_latched,
                        &mut self.out_clip_latched,
                    );
                }
                ActiveTab::VoiceStudio => {
                    render_studio(ui, self.active_preset_mut(), &mut chain_modified);
                }
                ActiveTab::Presets => {
                    render_presets_view(
                        ui,
                        &mut self.presets,
                        &self.active_preset_id,
                        &mut self.search_query,
                        &mut select_preset,
                        &mut export_preset,
                        &mut import_requested,
                    );
                }
                ActiveTab::AudioSettings => {
                    let in_devs = self.audio_engine.device_manager().list_input_devices();
                    let out_devs = self.audio_engine.device_manager().list_output_devices();

                    render_audio_settings(
                        ui,
                        &in_devs,
                        &out_devs,
                        &mut self.audio_engine.current_input_device,
                        &mut self.audio_engine.current_output_device,
                        &mut self.audio_engine.current_buffer_size,
                        &cable_stat,
                        &mut self.start_minimized,
                        &mut self.start_with_windows,
                        &mut device_changed,
                        &mut autostart_toggled,
                    );
                }
                ActiveTab::Diagnostics => {
                    render_diagnostics(
                        ui,
                        &self.telemetry,
                        &self.audio_engine.current_input_device,
                        &self.audio_engine.current_output_device,
                        &mut export_diagnostics,
                    );
                }
            }

            // Action dispatchers
            if fix_routing {
                // Auto-route to detected VB-CABLE or jump to audio settings
                if let Some(ref cable_out) = cable_stat.output_device_name {
                    self.audio_engine.current_output_device = cable_out.clone();
                    if self.audio_engine.is_running() {
                        self.toggle_engine();
                        self.toggle_engine();
                    }
                } else {
                    self.active_tab = ActiveTab::AudioSettings;
                }
            }
            if toggle_engine {
                self.toggle_engine();
            }
            if let Some(id) = select_preset {
                self.select_preset(id);
            }
            if toggle_monitor {
                self.monitoring_enabled = !self.monitoring_enabled;
                self.audio_engine.set_monitoring(self.monitoring_enabled);
            }
            if chain_modified {
                let p = self.active_preset().clone();
                if self.audio_engine.is_running() {
                    self.audio_engine.send_command(AudioCommand::SetPreset(p.clone()));
                }
                let _ = self.preset_storage.save_preset(&p);
            }
            if let Some(p) = export_preset {
                let filename = format!("{}.toml", p.name.to_lowercase().replace(' ', "_"));
                let dest = PathBuf::from(&filename);
                let _ = crate::presets::storage::PresetStorage::export_preset(&p, &dest);
            }
            if autostart_toggled {
                let _ = AutostartManager::set_autostart(self.start_with_windows);
            }
            if export_diagnostics {
                let diag = serde_json::json!({
                    "app": "VoiceForge",
                    "latency_ms": self.telemetry.estimated_latency_ms,
                    "cpu_load_pct": self.telemetry.cpu_load_pct,
                    "dropouts": self.telemetry.dropouts,
                    "sample_rate": self.telemetry.sample_rate,
                    "buffer_size": self.telemetry.buffer_size,
                    "input_device": self.audio_engine.current_input_device,
                    "output_device": self.audio_engine.current_output_device,
                });
                let _ = std::fs::write("voiceforge_diagnostics.json", diag.to_string());
            }
            if device_changed && self.audio_engine.is_running() {
                self.toggle_engine();
                self.toggle_engine();
            }
        });

        // Smart Repaint: Only request repaint if audio engine is running or pulsing
        if self.audio_engine.is_running() && !self.reduce_motion {
            ctx.request_repaint_after(std::time::Duration::from_millis(30));
        }
    }
}
