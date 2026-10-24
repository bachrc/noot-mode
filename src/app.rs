use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use eframe::egui::{self, Color32, ColorImage, TextureHandle, TextureOptions, ViewportCommand};

use crate::service::ServiceCli;

const TEXTURE_ON: &[u8] = include_bytes!("../assets/noot-on.png");
const TEXTURE_OFF: &[u8] = include_bytes!("../assets/noot-off.png");
const POLL_INTERVAL: Duration = Duration::from_secs(2);
const BACKGROUND: Color32 = Color32::from_rgb(0x14, 0x14, 0x1a);
const ERROR: Color32 = Color32::from_rgb(0xff, 0x6b, 0x6b);

pub struct SettingsApp {
    service: ServiceCli,
    active: bool,
    last_poll: Instant,
    on: TextureHandle,
    off: TextureHandle,
    error: Option<String>,
}

impl SettingsApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Result<Self> {
        let service = ServiceCli::default();
        let active = service.is_active().unwrap_or(false);
        Ok(Self {
            service,
            active,
            last_poll: Instant::now(),
            on: load_texture(&cc.egui_ctx, "noot-on", TEXTURE_ON)?,
            off: load_texture(&cc.egui_ctx, "noot-off", TEXTURE_OFF)?,
            error: None,
        })
    }

    fn poll(&mut self) {
        match self.service.is_active() {
            Ok(active) => {
                self.active = active;
                self.error = None;
            }
            Err(error) => self.error = Some(format!("{error:#}")),
        }
    }

    fn toggle(&mut self) {
        match self.service.toggle() {
            Ok(enabled) => {
                self.active = enabled;
                self.error = None;
            }
            Err(error) => self.error = Some(format!("{error:#}")),
        }
    }

    fn hint(&self) -> String {
        if let Some(error) = &self.error {
            return error.clone();
        }
        if self.active {
            return String::from("Noot actif — clic pour désactiver, clic droit pour fermer");
        }
        String::from("Noot inactif — clic pour activer, clic droit pour fermer")
    }
}

fn load_texture(ctx: &egui::Context, name: &str, bytes: &[u8]) -> Result<TextureHandle> {
    let image = image::load_from_memory(bytes)?.to_rgba8();
    let width = usize::try_from(image.width()).map_err(|_| anyhow!("largeur d'image invalide"))?;
    let height = usize::try_from(image.height()).map_err(|_| anyhow!("hauteur d'image invalide"))?;
    let pixels = ColorImage::from_rgba_unmultiplied([width, height], image.as_raw());
    Ok(ctx.load_texture(name, pixels, TextureOptions::LINEAR))
}

impl eframe::App for SettingsApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.last_poll.elapsed() >= POLL_INTERVAL {
            self.poll();
            self.last_poll = Instant::now();
        }
        ctx.request_repaint_after(POLL_INTERVAL);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let bounds = ui.max_rect();
        let texture = if self.active { &self.on } else { &self.off };
        let natural = texture.size_vec2();
        let scale = (bounds.width() / natural.x).min(bounds.height() / natural.y);
        let image_rect = egui::Rect::from_center_size(bounds.center(), natural * scale);

        ui.painter().rect_filled(bounds, 0.0, BACKGROUND);
        let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
        ui.painter().image(texture.id(), image_rect, uv, Color32::WHITE);

        let response = ui.interact(bounds, ui.scope_id().with("noot-toggle"), egui::Sense::click_and_drag());
        let response = response.on_hover_text(self.hint());

        if response.clicked() {
            self.toggle();
            self.last_poll = Instant::now();
        }
        if response.drag_started() {
            ui.ctx().send_viewport_cmd(ViewportCommand::StartDrag);
        }
        if response.secondary_clicked() {
            ui.ctx().send_viewport_cmd(ViewportCommand::Close);
        }
        if self.error.is_some() {
            ui.painter()
                .rect_stroke(bounds, 0.0, egui::Stroke::new(2.0, ERROR), egui::StrokeKind::Inside);
        }
    }
}
