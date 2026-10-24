use std::sync::mpsc;
use std::time::{Duration, Instant};

use anyhow::Result;
use noot::audio::{AudioOutput, RateGate};
use noot::keys;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

const VOLUME: f32 = 0.7;
const MIN_INTERVAL: Duration = Duration::from_millis(90);
const OPEN_RETRY: Duration = Duration::from_secs(2);

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let mut audio = open_audio();
    let (presses, events) = mpsc::channel();
    keys::spawn_watcher(presses)?;

    let mut gate = RateGate::new(MIN_INTERVAL);
    info!("nootd prêt");
    for () in events {
        if gate.allow(Instant::now()) {
            audio.play();
        }
    }
    Ok(())
}

fn open_audio() -> AudioOutput {
    loop {
        match AudioOutput::open(VOLUME) {
            Ok(audio) => return audio,
            Err(error) => {
                warn!(%error, "périphérique audio indisponible, nouvelle tentative");
                std::thread::sleep(OPEN_RETRY);
            }
        }
    }
}
