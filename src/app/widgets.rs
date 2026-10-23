use std::f32::consts::PI;
use egui::{epaint::PathShape, Align2, Color32, CornerRadius, FontId, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, Widget};
use crate::app::theme::{ThemeColors, ThemeMetrics};

/// Large 88px Precision Rotary Knob with Drag, Scroll Wheel, Double-Click Reset, and Value Entry
pub struct LargeKnob<'a> {
    id_source: &'a str,
    label: &'a str,
    value: &'a mut f32,
    default_val: f32,
    min: f32,
    max: f32,
    unit: &'a str,
    size: f32,
}

impl<'a> LargeKnob<'a> {
    pub fn new(id_source: &'a str, label: &'a str, value: &'a mut f32, default_val: f32, min: f32, max: f32, unit: &'a str) -> Self {
        Self {
            id_source,
            label,
            value,
            default_val,
            min,
            max,
            unit,
            size: 88.0,
        }
    }
}

impl<'a> Widget for LargeKnob<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let knob_size = Vec2::splat(self.size);
        let id = ui.make_persistent_id(self.id_source);

        let (rect, mut response) = ui.allocate_exact_size(knob_size, Sense::click_and_drag());
        let range = self.max - self.min;

        // Double-click to reset
        if response.double_clicked() {
            *self.value = self.default_val;
            response.mark_changed();
        }

        // Drag handling
        if response.dragged() {
            let delta = -response.drag_delta().y + response.drag_delta().x;
            let step = range / 200.0;
            *self.value = (*self.value + delta * step).clamp(self.min, self.max);
            response.mark_changed();
        }

        // Scroll wheel handling
        if response.hovered() {
            let scroll_delta = ui.input(|i| i.raw_scroll_delta.y);
            if scroll_delta.abs() > 0.0 {
                let step = range / 100.0;
                *self.value = (*self.value + (scroll_delta.signum() * step)).clamp(self.min, self.max);
                response.mark_changed();
            }
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let center = rect.center();
            let r = self.size * 0.5 - 6.0;

            // Hover state animation
            let hover_anim = ui.ctx().animate_bool_with_time(id.with("hover"), response.hovered(), 0.12);
            let active_bg = if response.dragged() {
                ThemeColors::SURFACE_ACTIVE
            } else {
                Color32::from_rgb(
                    (ThemeColors::SURFACE.r() as f32 * (1.0 - hover_anim) + ThemeColors::SURFACE_RAISED.r() as f32 * hover_anim) as u8,
                    (ThemeColors::SURFACE.g() as f32 * (1.0 - hover_anim) + ThemeColors::SURFACE_RAISED.g() as f32 * hover_anim) as u8,
                    (ThemeColors::SURFACE.b() as f32 * (1.0 - hover_anim) + ThemeColors::SURFACE_RAISED.b() as f32 * hover_anim) as u8,
                )
            };

            // Knob outer background & subtle boundary
            painter.circle_filled(center, r, active_bg);
            let border_color = if response.dragged() { ThemeColors::BORDER_FOCUS } else { ThemeColors::BORDER };
            painter.circle_stroke(center, r, Stroke::new(1.0, border_color));

            // Track & Value Sweep Arc (135 deg to 405 deg)
            let start_rad = 135.0_f32.to_radians();
            let sweep_rad = 270.0_f32.to_radians();
            let norm_val = ((*self.value - self.min) / range).clamp(0.0, 1.0);
            let current_rad = start_rad + sweep_rad * norm_val;

            // Background track arc
            let arc_r = r - 5.0;
            let steps = 36;
            let mut track_points = Vec::with_capacity(steps + 1);
            for i in 0..=steps {
                let a = start_rad + sweep_rad * (i as f32 / steps as f32);
                track_points.push(center + Vec2::new(a.cos(), a.sin()) * arc_r);
            }
            painter.add(PathShape::line(track_points, Stroke::new(3.0, ThemeColors::BORDER)));

            // Active value arc
            let active_steps = (steps as f32 * norm_val).ceil() as usize;
            if active_steps > 0 {
                let mut active_points = Vec::with_capacity(active_steps + 1);
                for i in 0..=active_steps {
                    let a = start_rad + sweep_rad * ((i as f32 / steps as f32).min(norm_val));
                    active_points.push(center + Vec2::new(a.cos(), a.sin()) * arc_r);
                }
                painter.add(PathShape::line(active_points, Stroke::new(3.0, ThemeColors::ACCENT)));
            }

            // Central core cap
            let core_r = r * 0.58;
            painter.circle_filled(center, core_r, ThemeColors::BG);
            painter.circle_stroke(center, core_r, Stroke::new(1.0, ThemeColors::BORDER));

            // Radial pointer needle
            let needle_inner = center + Vec2::new(current_rad.cos(), current_rad.sin()) * (core_r * 0.4);
            let needle_outer = center + Vec2::new(current_rad.cos(), current_rad.sin()) * (r - 9.0);
            painter.line_segment([needle_inner, needle_outer], Stroke::new(2.5, ThemeColors::ACCENT_SOFT));
        }

        // Value text & Label below the knob
        ui.vertical_centered(|ui| {
            ui.add_space(4.0);
            let sign = if *self.value > 0.05 && self.unit != "%" { "+" } else { "" };
            let readout = format!("{}{:.1} {}", sign, *self.value, self.unit);
            ui.label(egui::RichText::new(readout).font(FontId::monospace(13.0)).strong().color(ThemeColors::TEXT));
            ui.label(egui::RichText::new(self.label.to_uppercase()).font(FontId::proportional(11.0)).color(ThemeColors::TEXT_MUTED));
        });

        response
    }
}

/// Medium Rotary Knob for Studio Panels & Effect Chains
pub struct RotaryKnob<'a> {
    label: &'a str,
    value: &'a mut f32,
    min: f32,
    max: f32,
    unit: &'a str,
    radius: f32,
}

impl<'a> RotaryKnob<'a> {
    pub fn new(label: &'a str, value: &'a mut f32, min: f32, max: f32, unit: &'a str) -> Self {
        Self {
            label,
            value,
            min,
            max,
            unit,
            radius: 20.0,
        }
    }

    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }
}

impl<'a> Widget for RotaryKnob<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let size = Vec2::splat(self.radius * 2.0 + 8.0);
        let (rect, mut response) = ui.allocate_exact_size(size, Sense::click_and_drag());
        let range = self.max - self.min;

        if response.dragged() {
            let drag_delta = -response.drag_delta().y + response.drag_delta().x;
            let step = range / 200.0;
            *self.value = (*self.value + drag_delta * step).clamp(self.min, self.max);
            response.mark_changed();
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let center = rect.center();
            let r = self.radius;

            painter.circle_filled(center, r, ThemeColors::SURFACE);
            painter.circle_stroke(center, r, Stroke::new(1.0, ThemeColors::BORDER));

            let start_angle = 135.0_f32.to_radians();
            let sweep_range = 270.0_f32.to_radians();
            let norm_val = ((*self.value - self.min) / range).clamp(0.0, 1.0);
            let current_angle = start_angle + sweep_range * norm_val;

            // Track background arc
            let steps = 28;
            let mut track_points = Vec::with_capacity(steps + 1);
            for i in 0..=steps {
                let angle = start_angle + sweep_range * (i as f32 / steps as f32);
                track_points.push(center + Vec2::new(angle.cos(), angle.sin()) * (r - 4.0));
            }
            painter.add(PathShape::line(track_points, Stroke::new(2.5, ThemeColors::BORDER)));

            // Value arc
            let active_steps = (steps as f32 * norm_val).ceil() as usize;
            if active_steps > 0 {
                let mut active_points = Vec::with_capacity(active_steps + 1);
                for i in 0..=active_steps {
                    let angle = start_angle + sweep_range * ((i as f32 / steps as f32).min(norm_val));
                    active_points.push(center + Vec2::new(angle.cos(), angle.sin()) * (r - 4.0));
                }
                painter.add(PathShape::line(active_points, Stroke::new(2.5, ThemeColors::ACCENT)));
            }

            // Radial line
            let needle_inner = center + Vec2::new(current_angle.cos(), current_angle.sin()) * (r * 0.25);
            let needle_outer = center + Vec2::new(current_angle.cos(), current_angle.sin()) * (r - 5.0);
            painter.line_segment([needle_inner, needle_outer], Stroke::new(2.0, ThemeColors::ACCENT_SOFT));
        }

        ui.vertical(|ui| {
            ui.label(egui::RichText::new(self.label).font(FontId::proportional(11.0)).color(ThemeColors::TEXT_MUTED));
            ui.label(egui::RichText::new(format!("{:.1} {}", *self.value, self.unit)).font(FontId::monospace(11.0)).color(ThemeColors::TEXT));
        });

        response
    }
}

/// Horizontal Segmented Stereo Meter with Latching Clip Indicator & dB Scale
pub struct SegmentedMeter<'a> {
    id_source: &'a str,
    level_db: f32,
    peak_db: f32,
    clip_latched: &'a mut bool,
    width: f32,
    height: f32,
}

impl<'a> SegmentedMeter<'a> {
    pub fn new(id_source: &'a str, level_db: f32, peak_db: f32, clip_latched: &'a mut bool) -> Self {
        Self {
            id_source,
            level_db,
            peak_db,
            clip_latched,
            width: 220.0,
            height: 16.0,
        }
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
}

impl<'a> Widget for SegmentedMeter<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(Vec2::new(self.width, self.height), Sense::click());

        // Latch clip if signal peaks above 0.0 dBFS
        if self.peak_db >= 0.0 {
            *self.clip_latched = true;
        }

        // Clicking the meter resets the clip latch
        if response.clicked() {
            *self.clip_latched = false;
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Background trough
            painter.rect_filled(rect, CornerRadius::same(3), ThemeColors::BG);
            painter.rect_stroke(rect, CornerRadius::same(3), Stroke::new(1.0, ThemeColors::BORDER), egui::StrokeKind::Inside);

            let min_db = -60.0f32;
            let max_db = 3.0f32;
            let range = max_db - min_db;

            // Clip button rect at rightmost edge
            let clip_width = 24.0;
            let meter_width = rect.width() - clip_width - 4.0;

            // Discrete LED segments
            let num_segments = 28;
            let seg_gap = 1.5;
            let seg_width = (meter_width - (seg_gap * (num_segments - 1) as f32)) / num_segments as f32;

            for i in 0..num_segments {
                let seg_frac = (i + 1) as f32 / num_segments as f32;
                let seg_db = min_db + seg_frac * range;

                let is_lit = self.level_db >= seg_db && self.level_db > -59.5;

                // Color zoning: Green (-60 to -12), Amber (-12 to 0), Red (0 to +3)
                let zone_color = if seg_db > 0.0 {
                    ThemeColors::DANGER
                } else if seg_db > -12.0 {
                    ThemeColors::WARN
                } else {
                    ThemeColors::SUCCESS
                };

                let color = if is_lit {
                    zone_color
                } else {
                    Color32::from_rgba_premultiplied(zone_color.r() / 5, zone_color.g() / 5, zone_color.b() / 5, 45)
                };

                let seg_x = rect.min.x + 2.0 + (i as f32 * (seg_width + seg_gap));
                let seg_rect = Rect::from_min_max(
                    Pos2::new(seg_x, rect.min.y + 2.0),
                    Pos2::new(seg_x + seg_width, rect.max.y - 2.0),
                );
                painter.rect_filled(seg_rect, CornerRadius::same(1), color);
            }

            // Peak Hold line
            if self.peak_db > -59.5 {
                let peak_norm = ((self.peak_db - min_db) / range).clamp(0.0, 1.0);
                let peak_x = rect.min.x + 2.0 + (meter_width * peak_norm);
                painter.line_segment(
                    [Pos2::new(peak_x, rect.min.y + 1.0), Pos2::new(peak_x, rect.max.y - 1.0)],
                    Stroke::new(1.5, ThemeColors::ACCENT_SOFT),
                );
            }

            // Clip Indicator Box
            let clip_rect = Rect::from_min_max(
                Pos2::new(rect.max.x - clip_width, rect.min.y + 1.0),
                Pos2::new(rect.max.x - 1.0, rect.max.y - 1.0),
            );

            let clip_bg = if *self.clip_latched {
                ThemeColors::CLIP_LATCH
            } else {
                ThemeColors::SURFACE
            };
            painter.rect_filled(clip_rect, CornerRadius::same(2), clip_bg);
            painter.rect_stroke(clip_rect, CornerRadius::same(2), Stroke::new(1.0, ThemeColors::BORDER), egui::StrokeKind::Inside);

            let clip_text_color = if *self.clip_latched { Color32::WHITE } else { ThemeColors::TEXT_FAINT };
            painter.text(
                clip_rect.center(),
                Align2::CENTER_CENTER,
                "CLIP",
                FontId::monospace(9.0),
                clip_text_color,
            );
        }

        response
    }
}

/// Live Waveform & Spectrum Visualizer Strip
pub fn render_waveform_strip(ui: &mut Ui, is_running: bool, level_db: f32, width: f32, height: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        painter.rect_filled(rect, CornerRadius::same(ThemeMetrics::RADIUS_CONTROL as u8), ThemeColors::BG);
        painter.rect_stroke(rect, CornerRadius::same(ThemeMetrics::RADIUS_CONTROL as u8), Stroke::new(1.0, ThemeColors::BORDER), egui::StrokeKind::Inside);

        let center_y = rect.center().y;
        let time = ui.ctx().input(|i| i.time as f32);

        if is_running && level_db > -55.0 {
            let points_count = 64;
            let amp = ((level_db + 60.0) / 63.0).clamp(0.05, 1.0) * (height * 0.42);
            let mut points = Vec::with_capacity(points_count);

            for i in 0..points_count {
                let frac = i as f32 / (points_count - 1) as f32;
                let x = rect.min.x + frac * rect.width();
                // Multi-harmonic synthesized movement
                let wave1 = (frac * 18.0 + time * 12.0).sin();
                let wave2 = (frac * 36.0 - time * 16.0).cos() * 0.4;
                let wave3 = (frac * 6.0 + time * 6.0).sin() * 0.5;
                let y = center_y + (wave1 + wave2 + wave3) * amp * (0.8 + 0.2 * (frac * PI).sin());
                points.push(Pos2::new(x, y));
            }

            painter.add(PathShape::line(points, Stroke::new(1.5, ThemeColors::ACCENT_SOFT)));
        } else {
            // Idle flat hairline
            let p1 = Pos2::new(rect.min.x + 8.0, center_y);
            let p2 = Pos2::new(rect.max.x - 8.0, center_y);
            painter.line_segment([p1, p2], Stroke::new(1.0, ThemeColors::BORDER));
        }
    }
}

/// Custom Titlebar with drag region and custom window control buttons
pub fn render_titlebar(
    ui: &mut Ui,
    title: &str,
    on_minimize: &mut bool,
    on_maximize: &mut bool,
    on_close: &mut bool,
) {
    let height = ThemeMetrics::TITLEBAR_HEIGHT;
    let (rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::click_and_drag());

    if response.dragged() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        painter.rect_filled(rect, CornerRadius::ZERO, ThemeColors::BG);
        painter.line_segment(
            [Pos2::new(rect.min.x, rect.max.y), Pos2::new(rect.max.x, rect.max.y)],
            Stroke::new(1.0, ThemeColors::BORDER),
        );

        // App Emblem Icon (Crisp vector wave + diamond badge)
        let icon_center = Pos2::new(rect.min.x + 18.0, rect.center().y);
        painter.circle_filled(icon_center, 7.0, ThemeColors::ACCENT);
        painter.line_segment(
            [icon_center + Vec2::new(-4.0, 0.0), icon_center + Vec2::new(4.0, 0.0)],
            Stroke::new(2.0, Color32::WHITE),
        );

        // Title text
        painter.text(
            Pos2::new(rect.min.x + 34.0, rect.center().y),
            Align2::LEFT_CENTER,
            title,
            FontId::proportional(12.0),
            ThemeColors::TEXT,
        );

        // Window control buttons at right
        let btn_width = 36.0;
        let btn_height = height - 2.0;

        let close_rect = Rect::from_min_size(Pos2::new(rect.max.x - btn_width, rect.min.y + 1.0), Vec2::new(btn_width, btn_height));
        let max_rect = Rect::from_min_size(Pos2::new(rect.max.x - btn_width * 2.0, rect.min.y + 1.0), Vec2::new(btn_width, btn_height));
        let min_rect = Rect::from_min_size(Pos2::new(rect.max.x - btn_width * 3.0, rect.min.y + 1.0), Vec2::new(btn_width, btn_height));

        let mouse_pos = ui.ctx().input(|i| i.pointer.hover_pos().unwrap_or(Pos2::ZERO));

        // Minimize Button
        let min_hovered = min_rect.contains(mouse_pos);
        if min_hovered {
            painter.rect_filled(min_rect, CornerRadius::same(2), ThemeColors::SURFACE_HOVER);
            if ui.ctx().input(|i| i.pointer.primary_clicked()) {
                *on_minimize = true;
            }
        }
        painter.line_segment(
            [min_rect.center() + Vec2::new(-5.0, 0.0), min_rect.center() + Vec2::new(5.0, 0.0)],
            Stroke::new(1.2, ThemeColors::TEXT),
        );

        // Maximize Button
        let max_hovered = max_rect.contains(mouse_pos);
        if max_hovered {
            painter.rect_filled(max_rect, CornerRadius::same(2), ThemeColors::SURFACE_HOVER);
            if ui.ctx().input(|i| i.pointer.primary_clicked()) {
                *on_maximize = true;
            }
        }
        let max_box = Rect::from_center_size(max_rect.center(), Vec2::splat(10.0));
        painter.rect_stroke(max_box, CornerRadius::ZERO, Stroke::new(1.2, ThemeColors::TEXT), egui::StrokeKind::Inside);

        // Close Button
        let close_hovered = close_rect.contains(mouse_pos);
        if close_hovered {
            painter.rect_filled(close_rect, CornerRadius::same(2), ThemeColors::DANGER);
            if ui.ctx().input(|i| i.pointer.primary_clicked()) {
                *on_close = true;
            }
        }
        let c = close_rect.center();
        let stroke_col = if close_hovered { Color32::WHITE } else { ThemeColors::TEXT };
        painter.line_segment([c + Vec2::new(-4.0, -4.0), c + Vec2::new(4.0, 4.0)], Stroke::new(1.2, stroke_col));
        painter.line_segment([c + Vec2::new(-4.0, 4.0), c + Vec2::new(4.0, -4.0)], Stroke::new(1.2, stroke_col));
    }
}

/// Voicemod-Style Compact Rail Nav Item with Vector Icon and Active Glow
pub fn render_nav_item(ui: &mut Ui, label: &str, is_active: bool, icon_type: usize) -> Response {
    let width = ui.available_width();
    let is_compact = width < 100.0;
    let height = if is_compact { 46.0 } else { 38.0 };
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();

        let bg = if is_active {
            ThemeColors::SURFACE_ACTIVE
        } else if response.hovered() {
            ThemeColors::SURFACE_HOVER
        } else {
            Color32::TRANSPARENT
        };

        painter.rect_filled(rect, CornerRadius::same(ThemeMetrics::RADIUS_CONTROL as u8), bg);

        if is_active {
            // Neon Cyan active indicator pill on left
            let bar_rect = Rect::from_min_max(
                Pos2::new(rect.min.x + 2.0, rect.min.y + 8.0),
                Pos2::new(rect.min.x + 5.0, rect.max.y - 8.0),
            );
            painter.rect_filled(bar_rect, CornerRadius::same(2), ThemeColors::ACCENT);
        }

        // Draw 18px vector icon
        let icon_origin = if is_compact {
            rect.center()
        } else {
            Pos2::new(rect.min.x + 18.0, rect.center().y)
        };
        let icon_stroke = Stroke::new(1.8, if is_active { ThemeColors::ACCENT } else if response.hovered() { ThemeColors::TEXT } else { ThemeColors::TEXT_MUTED });

        match icon_type {
            0 => { // Voicebox / Home: Voicemod Grid
                let box_r = Rect::from_center_size(icon_origin, Vec2::splat(14.0));
                painter.rect_stroke(box_r, CornerRadius::same(2), icon_stroke, egui::StrokeKind::Inside);
                painter.circle_filled(icon_origin, 2.0, if is_active { ThemeColors::ACCENT } else { ThemeColors::TEXT_MUTED });
            }
            1 => { // Voicelab / Studio: Flask / Sliders
                painter.line_segment([icon_origin + Vec2::new(-6.0, -6.0), icon_origin + Vec2::new(-6.0, 6.0)], icon_stroke);
                painter.line_segment([icon_origin + Vec2::new(0.0, -6.0), icon_origin + Vec2::new(0.0, 6.0)], icon_stroke);
                painter.line_segment([icon_origin + Vec2::new(6.0, -6.0), icon_origin + Vec2::new(6.0, 6.0)], icon_stroke);
                painter.circle_filled(icon_origin + Vec2::new(-6.0, -2.0), 2.5, if is_active { ThemeColors::ACCENT } else { ThemeColors::TEXT_MUTED });
                painter.circle_filled(icon_origin + Vec2::new(0.0, 2.0), 2.5, if is_active { ThemeColors::ACCENT } else { ThemeColors::TEXT_MUTED });
                painter.circle_filled(icon_origin + Vec2::new(6.0, -3.0), 2.5, if is_active { ThemeColors::ACCENT } else { ThemeColors::TEXT_MUTED });
            }
            2 => { // Voice Library / Community: Waveform Sound
                painter.line_segment([icon_origin + Vec2::new(-7.0, 0.0), icon_origin + Vec2::new(-3.0, -6.0)], icon_stroke);
                painter.line_segment([icon_origin + Vec2::new(-3.0, -6.0), icon_origin + Vec2::new(2.0, 6.0)], icon_stroke);
                painter.line_segment([icon_origin + Vec2::new(2.0, 6.0), icon_origin + Vec2::new(7.0, 0.0)], icon_stroke);
            }
            3 => { // Audio & Devices: Headphone
                painter.circle_stroke(icon_origin - Vec2::new(0.0, 1.0), 6.5, icon_stroke);
                painter.rect_filled(Rect::from_center_size(icon_origin + Vec2::new(-6.0, 2.0), Vec2::new(3.0, 6.0)), CornerRadius::same(1), if is_active { ThemeColors::ACCENT } else { ThemeColors::TEXT_MUTED });
                painter.rect_filled(Rect::from_center_size(icon_origin + Vec2::new(6.0, 2.0), Vec2::new(3.0, 6.0)), CornerRadius::same(1), if is_active { ThemeColors::ACCENT } else { ThemeColors::TEXT_MUTED });
            }
            _ => { // Diagnostics: Heartbeat / Meter
                painter.line_segment([icon_origin + Vec2::new(-7.0, 0.0), icon_origin + Vec2::new(-3.0, 0.0)], icon_stroke);
                painter.line_segment([icon_origin + Vec2::new(-3.0, 0.0), icon_origin + Vec2::new(-1.0, -6.0)], icon_stroke);
                painter.line_segment([icon_origin + Vec2::new(-1.0, -6.0), icon_origin + Vec2::new(1.0, 6.0)], icon_stroke);
                painter.line_segment([icon_origin + Vec2::new(1.0, 6.0), icon_origin + Vec2::new(3.0, 0.0)], icon_stroke);
                painter.line_segment([icon_origin + Vec2::new(3.0, 0.0), icon_origin + Vec2::new(7.0, 0.0)], icon_stroke);
            }
        }

        if !is_compact {
            let text_color = if is_active { ThemeColors::TEXT } else { ThemeColors::TEXT_MUTED };
            painter.text(
                Pos2::new(rect.min.x + 40.0, rect.center().y),
                Align2::LEFT_CENTER,
                label,
                FontId::proportional(13.0),
                text_color,
            );
        }
    }

    response.on_hover_text(label)
}

/// Voicemod-Style Global HUD Toggle Pill (Voice Changer, Hear Myself, Mute Mic)
pub fn render_hud_toggle(
    ui: &mut Ui,
    _id_source: &str,
    label: &str,
    is_active: bool,
    active_color: Color32,
    icon_type: u8, // 0 = Voice Modulation Wave, 1 = Headphones/Ear (Hear Myself), 2 = Mic/Mute
) -> Response {
    let width = 132.0;
    let height = 32.0;
    let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let bg = if is_active {
            ThemeColors::SURFACE_ACTIVE
        } else if response.hovered() {
            ThemeColors::SURFACE_HOVER
        } else {
            ThemeColors::SURFACE
        };

        let border_color = if is_active { active_color } else { ThemeColors::BORDER };
        painter.rect_filled(rect, CornerRadius::same(ThemeMetrics::RADIUS_CONTROL as u8), bg);
        painter.rect_stroke(rect, CornerRadius::same(ThemeMetrics::RADIUS_CONTROL as u8), Stroke::new(1.0, border_color), egui::StrokeKind::Inside);

        // State indicator dot or vector icon
        let icon_x = rect.min.x + 16.0;
        let icon_y = rect.center().y;
        let icon_color = if is_active { active_color } else { ThemeColors::TEXT_MUTED };

        match icon_type {
            0 => { // Voice Modulation Wave
                painter.line_segment([Pos2::new(icon_x - 6.0, icon_y), Pos2::new(icon_x - 2.0, icon_y - 5.0)], Stroke::new(1.5, icon_color));
                painter.line_segment([Pos2::new(icon_x - 2.0, icon_y - 5.0), Pos2::new(icon_x + 2.0, icon_y + 5.0)], Stroke::new(1.5, icon_color));
                painter.line_segment([Pos2::new(icon_x + 2.0, icon_y + 5.0), Pos2::new(icon_x + 6.0, icon_y)], Stroke::new(1.5, icon_color));
            }
            1 => { // Headphones / Ear (Hear Myself)
                painter.circle_stroke(Pos2::new(icon_x, icon_y - 1.0), 5.0, Stroke::new(1.5, icon_color));
                painter.rect_filled(Rect::from_center_size(Pos2::new(icon_x - 5.0, icon_y + 1.0), Vec2::new(2.5, 5.0)), CornerRadius::same(1), icon_color);
                painter.rect_filled(Rect::from_center_size(Pos2::new(icon_x + 5.0, icon_y + 1.0), Vec2::new(2.5, 5.0)), CornerRadius::same(1), icon_color);
            }
            _ => { // Mic / Mute
                painter.rect_filled(Rect::from_center_size(Pos2::new(icon_x, icon_y - 2.0), Vec2::new(4.0, 7.0)), CornerRadius::same(2), icon_color);
                painter.line_segment([Pos2::new(icon_x, icon_y + 3.0), Pos2::new(icon_x, icon_y + 6.0)], Stroke::new(1.5, icon_color));
                if !is_active {
                    // Strike-through line when muted
                    painter.line_segment([Pos2::new(icon_x - 5.0, icon_y - 6.0), Pos2::new(icon_x + 5.0, icon_y + 6.0)], Stroke::new(1.5, ThemeColors::DANGER));
                }
            }
        }

        // Text label
        let text_color = if is_active { ThemeColors::TEXT } else { ThemeColors::TEXT_MUTED };
        painter.text(
            Pos2::new(rect.min.x + 30.0, rect.center().y),
            Align2::LEFT_CENTER,
            label,
            FontId::proportional(12.0),
            text_color,
        );

        // State pill on right [ON / OFF]
        let state_text = if is_active { "ON" } else { "OFF" };
        let state_col = if is_active { active_color } else { ThemeColors::TEXT_FAINT };
        painter.text(
            Pos2::new(rect.max.x - 12.0, rect.center().y),
            Align2::RIGHT_CENTER,
            state_text,
            FontId::monospace(10.0),
            state_col,
        );
    }

    response
}

/// Voicemod-Style Circular Voice Tile with Procedural Character Avatar
pub fn render_circular_voice_tile(
    ui: &mut Ui,
    preset_id: &str,
    name: &str,
    sound_tag: &str,
    hotkey: Option<&str>,
    is_active: bool,
    card_width: f32,
) -> Response {
    let circle_radius = 28.0; // 56px circular avatar
    let total_height = 98.0;
    let (rect, response) = ui.allocate_exact_size(Vec2::new(card_width, total_height), Sense::click());

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let center_x = rect.center().x;
        let avatar_center = Pos2::new(center_x, rect.min.y + circle_radius + 4.0);

        // Hover / Active Glow Ring
        if is_active {
            // Neon cyan outer glow
            painter.circle_stroke(avatar_center, circle_radius + 4.0, Stroke::new(3.0, ThemeColors::ACCENT));
            painter.circle_stroke(avatar_center, circle_radius + 1.5, Stroke::new(1.0, Color32::WHITE));
        } else if response.hovered() {
            painter.circle_stroke(avatar_center, circle_radius + 2.5, Stroke::new(2.0, Color32::from_rgb(0x3B, 0x82, 0xF6)));
        } else {
            painter.circle_stroke(avatar_center, circle_radius + 1.0, Stroke::new(1.0, ThemeColors::BORDER));
        }

        // Circular background base
        let base_bg = match preset_id {
            "demon" | "dark_voice" | "villain" => Color32::from_rgb(38, 14, 20),
            "alien" => Color32::from_rgb(10, 36, 26),
            "robot" | "vocoder_robot" => Color32::from_rgb(16, 28, 44),
            "cartoon" | "chipmunk" | "kid" => Color32::from_rgb(38, 30, 12),
            "natural" => Color32::from_rgb(14, 38, 32),
            "ghost" | "whisper" | "echo" => Color32::from_rgb(20, 26, 40),
            _ => ThemeColors::SURFACE_RAISED,
        };
        painter.circle_filled(avatar_center, circle_radius, base_bg);

        // Procedural Vector Avatar Artwork
        draw_procedural_avatar(painter, avatar_center, preset_id, is_active);

        // Favorite indicator dot
        if is_active {
            let badge_pos = avatar_center + Vec2::new(circle_radius * 0.7, circle_radius * 0.7);
            painter.circle_filled(badge_pos, 5.0, ThemeColors::ACCENT);
            painter.circle_filled(badge_pos, 2.5, Color32::from_rgb(10, 14, 22));
        }

        // Voice Name label
        let name_color = if is_active { ThemeColors::ACCENT } else if response.hovered() { Color32::WHITE } else { ThemeColors::TEXT };
        painter.text(
            Pos2::new(center_x, rect.min.y + 68.0),
            Align2::CENTER_CENTER,
            name,
            FontId::proportional(12.0),
            name_color,
        );

        // Hotkey badge or Sound Tag
        let sub_text = if let Some(hk) = hotkey {
            format!("[{}]", hk)
        } else {
            if sound_tag.len() > 16 {
                format!("{}...", &sound_tag[..14])
            } else {
                sound_tag.to_string()
            }
        };
        painter.text(
            Pos2::new(center_x, rect.min.y + 84.0),
            Align2::CENTER_CENTER,
            sub_text,
            FontId::proportional(10.0),
            ThemeColors::TEXT_MUTED,
        );
    }

    response
}

pub fn draw_procedural_avatar(painter: &egui::Painter, center: Pos2, preset_id: &str, is_active: bool) {
    let accent_col = if is_active { ThemeColors::ACCENT } else { ThemeColors::TEXT };
    
    match preset_id {
        "alien" => {
            // Neon Green Extraterrestrial Head (Voicemod iconic alien)
            let green = Color32::from_rgb(0x10, 0xB9, 0x81);
            // Oval head
            painter.circle_filled(center, 14.0, green);
            // Slanted black eyes
            painter.line_segment([center + Vec2::new(-6.0, -2.0), center + Vec2::new(-2.0, 1.0)], Stroke::new(3.0, Color32::BLACK));
            painter.line_segment([center + Vec2::new(6.0, -2.0), center + Vec2::new(2.0, 1.0)], Stroke::new(3.0, Color32::BLACK));
        }
        "demon" | "dark_voice" => {
            // Crimson Demon Horns & Visage
            let red = Color32::from_rgb(0xEF, 0x44, 0x44);
            // Left horn
            painter.line_segment([center + Vec2::new(-8.0, 2.0), center + Vec2::new(-11.0, -10.0)], Stroke::new(2.5, red));
            // Right horn
            painter.line_segment([center + Vec2::new(8.0, 2.0), center + Vec2::new(11.0, -10.0)], Stroke::new(2.5, red));
            // Glowing eyes
            painter.circle_filled(center + Vec2::new(-4.0, -1.0), 2.2, Color32::from_rgb(255, 120, 120));
            painter.circle_filled(center + Vec2::new(4.0, -1.0), 2.2, Color32::from_rgb(255, 120, 120));
            // Fang smirk
            painter.line_segment([center + Vec2::new(-4.0, 6.0), center + Vec2::new(4.0, 6.0)], Stroke::new(1.8, red));
        }
        "robot" | "vocoder_robot" => {
            // Cyber Android Visor
            let cyan = Color32::from_rgb(0x06, 0xB6, 0xD4);
            let box_r = Rect::from_center_size(center, Vec2::new(22.0, 18.0));
            painter.rect_stroke(box_r, CornerRadius::same(3), Stroke::new(1.8, cyan), egui::StrokeKind::Inside);
            // Visor line
            painter.line_segment([center + Vec2::new(-8.0, 0.0), center + Vec2::new(8.0, 0.0)], Stroke::new(3.0, ThemeColors::ACCENT));
            // Antenna
            painter.line_segment([center + Vec2::new(0.0, -9.0), center + Vec2::new(0.0, -14.0)], Stroke::new(1.8, cyan));
            painter.circle_filled(center + Vec2::new(0.0, -14.0), 2.0, ThemeColors::ACCENT);
        }
        "cartoon" | "chipmunk" | "kid" => {
            // Cheerful Wide-Eyed Toon
            let gold = Color32::from_rgb(0xFB, 0xBF, 0x24);
            painter.circle_stroke(center, 14.0, Stroke::new(1.8, gold));
            // Eyes
            painter.circle_filled(center + Vec2::new(-5.0, -3.0), 2.5, Color32::WHITE);
            painter.circle_filled(center + Vec2::new(5.0, -3.0), 2.5, Color32::WHITE);
            // Big smile curve
            painter.line_segment([center + Vec2::new(-6.0, 4.0), center + Vec2::new(0.0, 8.0)], Stroke::new(2.0, gold));
            painter.line_segment([center + Vec2::new(0.0, 8.0), center + Vec2::new(6.0, 4.0)], Stroke::new(2.0, gold));
        }
        "natural" => {
            // Clean Studio Broadcast Mic (Clean Mic badge!)
            let teal = ThemeColors::ACCENT;
            // Mic capsule
            painter.rect_filled(Rect::from_center_size(center - Vec2::new(0.0, 4.0), Vec2::new(10.0, 14.0)), CornerRadius::same(5), teal);
            // Pop filter ring
            painter.circle_stroke(center - Vec2::new(0.0, 4.0), 9.0, Stroke::new(1.5, Color32::WHITE));
            // Stand
            painter.line_segment([center + Vec2::new(0.0, 5.0), center + Vec2::new(0.0, 12.0)], Stroke::new(2.0, teal));
            painter.line_segment([center + Vec2::new(-6.0, 12.0), center + Vec2::new(6.0, 12.0)], Stroke::new(2.0, teal));
        }
        "echo" | "hall" | "cave" | "stadium" => {
            // Expanding Sonar Echo Waves
            let blue = Color32::from_rgb(0x38, 0xBD, 0xF8);
            painter.circle_stroke(center, 5.0, Stroke::new(1.5, blue));
            painter.circle_stroke(center, 10.0, Stroke::new(1.5, blue));
            painter.circle_stroke(center, 15.0, Stroke::new(1.5, blue));
        }
        "radio" | "telephone" | "megaphone" | "walkie_talkie" => {
            // Vintage Communication Device
            let orange = Color32::from_rgb(0xF9, 0x73, 0x16);
            painter.rect_stroke(Rect::from_center_size(center, Vec2::new(14.0, 20.0)), CornerRadius::same(2), Stroke::new(1.8, orange), egui::StrokeKind::Inside);
            // Speaker grill lines
            painter.line_segment([center + Vec2::new(-4.0, -3.0), center + Vec2::new(4.0, -3.0)], Stroke::new(1.5, orange));
            painter.line_segment([center + Vec2::new(-4.0, 0.0), center + Vec2::new(4.0, 0.0)], Stroke::new(1.5, orange));
            painter.line_segment([center + Vec2::new(-4.0, 3.0), center + Vec2::new(4.0, 3.0)], Stroke::new(1.5, orange));
        }
        "ghost" | "whisper" => {
            // Phantom Ethereal Cloak
            let violet = Color32::from_rgb(0xC0, 0x84, 0xFC);
            painter.circle_filled(center - Vec2::new(0.0, 3.0), 10.0, violet);
            painter.circle_filled(center + Vec2::new(-3.0, -4.0), 2.0, Color32::from_rgb(20, 20, 35));
            painter.circle_filled(center + Vec2::new(3.0, -4.0), 2.0, Color32::from_rgb(20, 20, 35));
            // Floating tail
            painter.line_segment([center + Vec2::new(-7.0, 6.0), center + Vec2::new(-3.0, 11.0)], Stroke::new(2.0, violet));
            painter.line_segment([center + Vec2::new(0.0, 6.0), center + Vec2::new(2.0, 12.0)], Stroke::new(2.0, violet));
            painter.line_segment([center + Vec2::new(7.0, 6.0), center + Vec2::new(5.0, 10.0)], Stroke::new(2.0, violet));
        }
        _ => {
            // Sleek Sound Waveform Glyph
            painter.line_segment([center + Vec2::new(-10.0, 0.0), center + Vec2::new(-5.0, -9.0)], Stroke::new(2.0, accent_col));
            painter.line_segment([center + Vec2::new(-5.0, -9.0), center + Vec2::new(0.0, 8.0)], Stroke::new(2.0, accent_col));
            painter.line_segment([center + Vec2::new(0.0, 8.0), center + Vec2::new(5.0, -7.0)], Stroke::new(2.0, accent_col));
            painter.line_segment([center + Vec2::new(5.0, -7.0), center + Vec2::new(10.0, 0.0)], Stroke::new(2.0, accent_col));
        }
    }
}


