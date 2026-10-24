use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use anyhow::Result;
use evdev::{Device, EventSummary, InputEvent, KeyCode};
use tracing::{debug, info, warn};

const RESCAN_INTERVAL: Duration = Duration::from_secs(2);

pub fn spawn_watcher(presses: Sender<()>) -> Result<()> {
    let registered: Arc<Mutex<HashSet<PathBuf>>> = Arc::new(Mutex::new(HashSet::new()));
    thread::spawn(move || watch(&presses, &registered));
    Ok(())
}

fn watch(presses: &Sender<()>, registered: &Arc<Mutex<HashSet<PathBuf>>>) {
    let mut warned = false;
    loop {
        let devices: Vec<(PathBuf, Device)> = evdev::enumerate().collect();
        if devices.is_empty() && !warned {
            warn!("aucun périphérique lisible sous /dev/input : groupe input ou règle udev manquant");
            warned = true;
        }
        for (path, device) in devices {
            if !is_keyboard(&device) {
                continue;
            }
            let Ok(mut guard) = registered.lock() else {
                return;
            };
            if !guard.insert(path.clone()) {
                continue;
            }
            drop(guard);

            let sender = presses.clone();
            let active = Arc::clone(registered);
            let key = path.clone();
            thread::spawn(move || {
                read_device(device, &sender);
                if let Ok(mut guard) = active.lock() {
                    guard.remove(&key);
                }
            });
            info!(path = %path.display(), "clavier suivi");
        }
        thread::sleep(RESCAN_INTERVAL);
    }
}

fn read_device(mut device: Device, presses: &Sender<()>) {
    loop {
        match device.fetch_events() {
            Ok(events) => {
                for event in events {
                    if is_press(&event) {
                        let _ = presses.send(());
                    }
                }
            }
            Err(error) => {
                debug!(%error, "fin de lecture du périphérique");
                return;
            }
        }
    }
}

#[must_use]
pub fn is_keyboard(device: &Device) -> bool {
    device
        .supported_keys()
        .is_some_and(|keys| keys.contains(KeyCode::KEY_A) && keys.contains(KeyCode::KEY_ENTER))
}

#[must_use]
pub fn is_press(event: &InputEvent) -> bool {
    matches!(event.destructure(), EventSummary::Key(_, _, 1))
}
