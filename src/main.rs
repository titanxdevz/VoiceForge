#![allow(dead_code, unused_imports, deprecated, non_snake_case, clippy::too_many_arguments)]

mod app;
mod audio;
mod config;
mod dsp;
mod presets;
mod system;
mod tray;

use app::VoiceForgeApp;
use system::SingleInstanceGuard;

fn main() -> eframe::Result<()> {
    // 1. Single-instance enforcement
    let _guard = match SingleInstanceGuard::try_acquire("VoiceForge_SingleInstance_Mutex_v1") {
        Ok(guard) => guard,
        Err(_) => {
            SingleInstanceGuard::focus_existing_window("VoiceForge");
            return Ok(());
        }
    };

    // 2. Window configuration with custom frameless chrome
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("VoiceForge")
            .with_decorations(false)
            .with_inner_size([1040.0, 680.0])
            .with_min_inner_size([880.0, 560.0])
            .with_transparent(false)
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "VoiceForge",
        options,
        Box::new(|cc| Ok(Box::new(VoiceForgeApp::new(cc)))),
    )
}
