use muda::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

pub enum TrayAction {
    OpenWindow,
    ToggleEngine,
    QuickSelectPreset(String),
    ToggleMute,
    Quit,
}

pub struct SystemTrayManager {
    _tray_icon: TrayIcon,
    menu: Menu,
    pub open_id: muda::MenuId,
    pub toggle_id: muda::MenuId,
    pub mute_id: muda::MenuId,
    pub quit_id: muda::MenuId,
}

impl SystemTrayManager {
    pub fn new() -> Result<Self, String> {
        let menu = Menu::new();
        let open_item = MenuItem::new("Open VoiceForge", true, None);
        let toggle_item = MenuItem::new("Start Audio Engine", true, None);
        let mute_item = MenuItem::new("Mute Output", true, None);
        let quit_item = MenuItem::new("Quit", true, None);

        let open_id = open_item.id().clone();
        let toggle_id = toggle_item.id().clone();
        let mute_id = mute_item.id().clone();
        let quit_id = quit_item.id().clone();

        menu.append(&open_item).map_err(|e| e.to_string())?;
        menu.append(&toggle_item).map_err(|e| e.to_string())?;
        menu.append(&PredefinedMenuItem::separator()).map_err(|e| e.to_string())?;
        menu.append(&mute_item).map_err(|e| e.to_string())?;
        menu.append(&PredefinedMenuItem::separator()).map_err(|e| e.to_string())?;
        menu.append(&quit_item).map_err(|e| e.to_string())?;

        let icon = create_tray_icon_rgba(37, 99, 235); // Accent blue #2563EB
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu.clone()))
            .with_tooltip("VoiceForge Voice Changer")
            .with_icon(icon)
            .build()
            .map_err(|e| format!("Failed to create system tray icon: {}", e))?;

        Ok(Self {
            _tray_icon: tray_icon,
            menu,
            open_id,
            toggle_id,
            mute_id,
            quit_id,
        })
    }

    pub fn update_state(&self, is_running: bool, is_muted: bool) {
        if let Some(item) = self.menu.items().into_iter().find(|i| i.id() == &self.toggle_id)
            && let Some(menu_item) = item.as_menuitem() {
                let text = if is_running { "Stop Audio Engine" } else { "Start Audio Engine" };
                menu_item.set_text(text);
        }
        if let Some(item) = self.menu.items().into_iter().find(|i| i.id() == &self.mute_id)
            && let Some(menu_item) = item.as_menuitem() {
                let text = if is_muted { "Unmute Output" } else { "Mute Output" };
                menu_item.set_text(text);
        }
    }
}

fn create_tray_icon_rgba(r: u8, g: u8, b: u8) -> Icon {
    let width = 32;
    let height = 32;
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        for x in 0..width {
            let dx = (x as f32) - 16.0;
            let dy = (y as f32) - 16.0;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist <= 12.0 {
                // Circular icon with dark border
                if dist >= 10.0 {
                    rgba.extend_from_slice(&[20, 28, 43, 255]);
                } else {
                    rgba.extend_from_slice(&[r, g, b, 255]);
                }
            } else {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }

    Icon::from_rgba(rgba, width, height).unwrap_or_else(|_| Icon::from_rgba(vec![255; 32 * 32 * 4], 32, 32).unwrap())
}
