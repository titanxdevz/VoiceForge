use egui::{Color32, CornerRadius, Margin, Stroke, Style, TextStyle, Visuals};

pub struct ThemeColors;

impl ThemeColors {
    // Exact v2 Color System
    pub const BG: Color32 = Color32::from_rgb(0x0E, 0x10, 0x14);           // #0E1014 Main background
    pub const SIDEBAR: Color32 = Color32::from_rgb(0x0B, 0x0D, 0x10);      // #0B0D10 Left sidebar
    pub const SURFACE: Color32 = Color32::from_rgb(0x15, 0x19, 0x20);      // #151920 Card / panel surface
    pub const SURFACE_HOVER: Color32 = Color32::from_rgb(0x1B, 0x20, 0x29);// #1B2029 Hover state
    pub const SURFACE_RAISED: Color32 = Color32::from_rgb(0x1B, 0x20, 0x29);
    pub const SURFACE_ACTIVE: Color32 = Color32::from_rgb(0x1E, 0x24, 0x30);

    // Border: Subtle 1px
    pub const BORDER: Color32 = Color32::from_rgb(0x22, 0x28, 0x33);       // #222833 1px border
    pub const BORDER_FOCUS: Color32 = Color32::from_rgb(0x2D, 0xD4, 0xBF); // #2DD4BF Accent border

    // Text & Muted
    pub const TEXT: Color32 = Color32::from_rgb(0xE8, 0xEA, 0xED);         // #E8EAED High-contrast text
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x8B, 0x93, 0xA1);   // #8B93A1 Muted metadata
    pub const TEXT_FAINT: Color32 = Color32::from_rgb(0x56, 0x5E, 0x6C);

    // ONE Accent: #2DD4BF (Teal)
    pub const ACCENT: Color32 = Color32::from_rgb(0x2D, 0xD4, 0xBF);       // #2DD4BF Teal
    pub const ACCENT_SOFT: Color32 = Color32::from_rgb(0x5E, 0xEA, 0xD4);
    pub const ACCENT_BLUE: Color32 = Color32::from_rgb(0x2D, 0xD4, 0xBF);  // Map to Teal to preserve ONE accent rule
    pub const ACCENT_SUBTLE: Color32 = Color32::from_rgba_premultiplied(45, 212, 191, 24);

    // Danger: #F26D6D (Only for clipping and errors)
    pub const DANGER: Color32 = Color32::from_rgb(0xF2, 0x6D, 0x6D);       // #F26D6D
    pub const WARN: Color32 = Color32::from_rgb(0xE2, 0xA0, 0x3F);
    pub const SUCCESS: Color32 = Color32::from_rgb(0x2D, 0xD4, 0xBF);      // Follows ONE accent rule
    pub const CLIP_LATCH: Color32 = Color32::from_rgb(0xF2, 0x6D, 0x6D);

    // Aliases for compatibility
    pub const SECONDARY: Color32 = Self::ACCENT_SOFT;
    pub const SURFACE_BG: Color32 = Self::SURFACE;
    pub const BORDER_HAIRLINE: Color32 = Self::BORDER;
    pub const TEXT_PRIMARY: Color32 = Self::TEXT;
    pub const TEXT_SECONDARY: Color32 = Self::TEXT_MUTED;
    pub const METER_NORMAL: Color32 = Self::ACCENT;
    pub const METER_WARN: Color32 = Color32::from_rgb(0x38, 0xBD, 0xF8);
    pub const METER_CLIP: Color32 = Self::DANGER;
    pub const BG_DARK: Color32 = Self::BG;
}

pub struct ThemeMetrics;

impl ThemeMetrics {
    pub const GRID: f32 = 8.0;
    pub const RADIUS_PANEL: f32 = 12.0;    // 12 for cards
    pub const RADIUS_CONTROL: f32 = 8.0;   // 8 for controls
    pub const RADIUS_SMALL: f32 = 6.0;
    pub const RADIUS_PILL: f32 = 18.0;
    pub const RADIUS_AVATAR: f32 = 36.0;   // Full for avatars
    pub const BORDER_WIDTH: f32 = 1.0;
    pub const SIDEBAR_WIDTH: f32 = 200.0;  // 200px labeled sidebar, collapsible
    pub const TITLEBAR_HEIGHT: f32 = 36.0;
    pub const STATUSBAR_HEIGHT: f32 = 26.0;
    pub const BOTTOM_BAR_HEIGHT: f32 = 48.0; // 48px bottom bar
    pub const MASTER_DOCK_HEIGHT: f32 = 48.0;
}

pub fn apply_theme(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "phosphor".to_owned(),
        egui::FontData::from_static(egui_phosphor::Variant::Regular.font_bytes()).into(),
    );
    fonts.families.entry(egui::FontFamily::Proportional).or_default().push("phosphor".to_owned());
    ctx.set_fonts(fonts);

    let mut visuals = Visuals::dark();

    visuals.override_text_color = Some(ThemeColors::TEXT);
    visuals.panel_fill = ThemeColors::BG;
    visuals.window_fill = ThemeColors::SURFACE;
    visuals.faint_bg_color = ThemeColors::BG;
    visuals.extreme_bg_color = ThemeColors::BG;

    // Strict 1px hairline borders, no fuzzy shadows
    visuals.window_stroke = Stroke::new(ThemeMetrics::BORDER_WIDTH, ThemeColors::BORDER);
    visuals.window_corner_radius = CornerRadius::same(ThemeMetrics::RADIUS_PANEL as u8);

    visuals.widgets.noninteractive.bg_fill = ThemeColors::SURFACE;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(ThemeMetrics::BORDER_WIDTH, ThemeColors::BORDER);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(ThemeMetrics::BORDER_WIDTH, ThemeColors::TEXT_MUTED);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(ThemeMetrics::RADIUS_CONTROL as u8);

    visuals.widgets.inactive.bg_fill = ThemeColors::SURFACE;
    visuals.widgets.inactive.bg_stroke = Stroke::new(ThemeMetrics::BORDER_WIDTH, ThemeColors::BORDER);
    visuals.widgets.inactive.fg_stroke = Stroke::new(ThemeMetrics::BORDER_WIDTH, ThemeColors::TEXT);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(ThemeMetrics::RADIUS_CONTROL as u8);

    visuals.widgets.hovered.bg_fill = ThemeColors::SURFACE_HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::new(ThemeMetrics::BORDER_WIDTH, ThemeColors::ACCENT_SOFT);
    visuals.widgets.hovered.fg_stroke = Stroke::new(ThemeMetrics::BORDER_WIDTH, ThemeColors::TEXT);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(ThemeMetrics::RADIUS_CONTROL as u8);

    visuals.widgets.active.bg_fill = ThemeColors::SURFACE_ACTIVE;
    visuals.widgets.active.bg_stroke = Stroke::new(ThemeMetrics::BORDER_WIDTH, ThemeColors::ACCENT);
    visuals.widgets.active.fg_stroke = Stroke::new(ThemeMetrics::BORDER_WIDTH, ThemeColors::TEXT);
    visuals.widgets.active.corner_radius = CornerRadius::same(ThemeMetrics::RADIUS_CONTROL as u8);

    visuals.selection.bg_fill = ThemeColors::ACCENT;
    visuals.selection.stroke = Stroke::new(1.0, ThemeColors::ACCENT_SOFT);

    ctx.set_visuals(visuals);

    ctx.style_mut(|style| {
        style.spacing.item_spacing = egui::vec2(ThemeMetrics::GRID, ThemeMetrics::GRID);
        style.spacing.window_margin = Margin::same(12);
        style.spacing.button_padding = egui::vec2(10.0, 6.0);
    });
}
