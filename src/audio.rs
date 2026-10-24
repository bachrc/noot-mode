use std::io::Cursor;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow};
use rodio::buffer::SamplesBuffer;
use rodio::cpal::BufferSize;
use rodio::{ChannelCount, Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, SampleRate, Source};
use tracing::{info, warn};

const SOUNDS: [&[u8]; 2] = [
    include_bytes!("../assets/noot-first.wav"),
    include_bytes!("../assets/noot-second.wav"),
];
const BUFFER_CANDIDATES: [u32; 3] = [256, 512, 1024];

pub struct Sound {
    pub samples: Vec<f32>,
    pub channels: ChannelCount,
    pub sample_rate: SampleRate,
}

pub struct AudioOutput {
    sink: MixerDeviceSink,
    sounds: Vec<Sound>,
    rng: fastrand::Rng,
    volume: f32,
}

impl AudioOutput {
    pub fn open(volume: f32) -> Result<Self> {
        let sounds = SOUNDS.iter().map(|bytes| decode_wav(bytes)).collect::<Result<Vec<_>>>()?;
        let first = sounds.first().context("aucun son embarqué")?;
        let sink = open_sink(first.sample_rate)?;
        Ok(Self {
            sink,
            sounds,
            rng: fastrand::Rng::new(),
            volume,
        })
    }

    pub fn play(&mut self) {
        let index = random_index(&mut self.rng, self.sounds.len());
        let Some(sound) = self.sounds.get(index) else {
            return;
        };
        let source = SamplesBuffer::new(sound.channels, sound.sample_rate, sound.samples.clone()).amplify(self.volume);
        let player = Player::connect_new(self.sink.mixer());
        player.append(source);
        player.detach();
    }
}

#[must_use]
pub fn random_index(rng: &mut fastrand::Rng, count: usize) -> usize {
    if count == 0 { 0 } else { rng.usize(..count) }
}

pub fn decode_wav(bytes: &'static [u8]) -> Result<Sound> {
    let decoder = Decoder::new_wav(Cursor::new(bytes)).context("décodage du WAV")?;
    let channels = decoder.channels();
    let sample_rate = decoder.sample_rate();
    let samples: Vec<f32> = decoder.collect();
    if samples.is_empty() {
        return Err(anyhow!("son vide"));
    }
    Ok(Sound {
        samples,
        channels,
        sample_rate,
    })
}

fn open_sink(sample_rate: SampleRate) -> Result<MixerDeviceSink> {
    for frames in BUFFER_CANDIDATES {
        if let Ok(sink) = build(sample_rate, BufferSize::Fixed(frames)) {
            info!(frames, "périphérique audio ouvert en buffer fixe");
            return Ok(sink);
        }
    }
    let sink = build(sample_rate, BufferSize::Default).context("ouverture du flux audio")?;
    warn!("buffer audio par défaut : latence plus élevée");
    Ok(sink)
}

fn build(sample_rate: SampleRate, buffer_size: BufferSize) -> Result<MixerDeviceSink> {
    DeviceSinkBuilder::from_default_device()
        .context("aucun périphérique de sortie")?
        .with_sample_rate(sample_rate)
        .with_buffer_size(buffer_size)
        .open_stream()
        .map_err(|error| anyhow!("flux audio refusé : {error}"))
}

pub struct RateGate {
    interval: Duration,
    last: Option<Instant>,
}

impl RateGate {
    #[must_use]
    pub const fn new(interval: Duration) -> Self {
        Self { interval, last: None }
    }

    pub fn allow(&mut self, now: Instant) -> bool {
        match self.last {
            Some(last) if now.saturating_duration_since(last) < self.interval => false,
            _ => {
                self.last = Some(now);
                true
            }
        }
    }
}
