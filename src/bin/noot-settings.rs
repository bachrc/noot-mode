use eframe::egui;
use noot::app::SettingsApp;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([240.0, 240.0])
            .with_resizable(false)
            .with_decorations(false)
            .with_title("Noot mode"),
        ..Default::default()
    };
    eframe::run_native("noot-settings", options, Box::new(|cc| Ok(Box::new(SettingsApp::new(cc)?))))
}
