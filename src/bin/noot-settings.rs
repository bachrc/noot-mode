use eframe::egui;
use noot::app::SettingsApp;
use noot::input_check::{InputStatus, probe};

fn main() -> eframe::Result {
    if std::env::args().any(|argument| argument == "--version") {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if std::env::args().any(|argument| argument == "--check") {
        std::process::exit(check_input());
    }

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

fn check_input() -> i32 {
    match probe().unwrap_or(InputStatus::NoKeyboard) {
        InputStatus::Ready => {
            println!("clavier lisible");
            0
        }
        InputStatus::Blocked => {
            println!("clavier présent mais illisible : groupe input manquant");
            3
        }
        InputStatus::NoKeyboard => {
            println!("aucun clavier détecté");
            2
        }
    }
}
