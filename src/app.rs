use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use eframe::egui::{self, Color32, ColorImage, TextureHandle, TextureOptions, ViewportCommand};

use crate::input_check::{self, InputStatus};
use crate::service::ServiceCli;

const TEXTURE_ON: &[u8] = include_bytes!("../assets/noot-on.png");
const TEXTURE_OFF: &[u8] = include_bytes!("../assets/noot-off.png");
const POLL_INTERVAL: Duration = Duration::from_secs(2);
const BACKGROUND: Color32 = Color32::from_rgb(0x14, 0x14, 0x1a);
const ERROR_BAND: Color32 = Color32::from_rgb(0x2a, 0x0f, 0x12);
const ERROR_TEXT: Color32 = Color32::from_rgb(0xff, 0x6b, 0x6b);
const WARNING_BAND: Color32 = Color32::from_rgb(0x2a, 0x1f, 0x0a);
const WARNING_TEXT: Color32 = Color32::from_rgb(0xf5, 0xa6, 0x23);
const BAND_HEIGHT: f32 = 28.0;

pub struct SettingsApp {
    service: ServiceCli,
    active: bool,
    last_poll: Instant,
    on: TextureHandle,
    off: TextureHandle,
    error: Option<String>,
    input_status: InputStatus,
}

impl SettingsApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Result<Self> {
        let service = ServiceCli::default();
        let active = service.is_active().unwrap_or(false);
        let input_status = input_check::probe().unwrap_or(InputStatus::NoKeyboard);
        Ok(Self {
            service,
            active,
            last_poll: Instant::now(),
            on: load_texture(&cc.egui_ctx, "noot-on", TEXTURE_ON)?,
            off: load_texture(&cc.egui_ctx, "noot-off", TEXTURE_OFF)?,
            error: None,
            input_status,
        })
    }

    fn poll(&mut self) {
        match self.service.is_active() {
            Ok(active) => {
                self.active = active;
                self.error = None;
            }
            Err(error) => {
                let message = format!("{error:#}");
                eprintln!("{message}");
                self.error = Some(message);
            }
        }
    }

    fn toggle(&mut self) {
        let daemon = std::env::current_exe().map(|exe| exe.with_file_name("nootd"));
        if let Ok(daemon) = daemon {
            if let Err(error) = self.service.install_unit(&daemon) {
                let message = format!("{error:#}");
                eprintln!("{message}");
                self.error = Some(message);
                return;
            }
        }
        match self.service.toggle() {
            Ok(enabled) => {
                self.active = enabled;
                self.error = None;
            }
            Err(error) => {
                let message = format!("{error:#}");
                eprintln!("{message}");
                self.error = Some(message);
            }
        }
        self.input_status = input_check::probe().unwrap_or(self.input_status);
    }

    fn banner(&self) -> Option<(&'static str, Color32, Color32)> {
        if self.error.is_some() {
            return Some(("Échec systemctl — lancer noot-settings --check en terminal", ERROR_BAND, ERROR_TEXT));
        }
        if self.input_status == InputStatus::Blocked {
            return Some((
                "Clavier illisible — sudo usermod -aG input $USER, puis se reconnecter",
                WARNING_BAND,
                WARNING_TEXT,
            ));
        }
        None
    }
}

fn load_texture(ctx: &egui::Context, name: &str, bytes: &[u8]) -> Result<TextureHandle> {
    let image = image::load_from_memory(bytes)?.to_rgba8();
    let width = usize::try_from(image.width()).map_err(|_| anyhow!("largeur d'image invalide"))?;
    let height = usize::try_from(image.height()).map_err(|_| anyhow!("hauteur d'image invalide"))?;
    let pixels = ColorImage::from_rgba_unmultiplied([width, height], image.as_raw());
    Ok(ctx.load_texture(name, pixels, TextureOptions::LINEAR))
}

fn paint_marquee(ui: &egui::Ui, band: egui::Rect, text: &str, color: Color32) {
    let font = egui::FontId::proportional(12.0);
    let galley = ui.painter().layout_no_wrap(text.to_owned(), font, color);
    let text_width = galley.size().x;
    let top = band.center().y - galley.size().y / 2.0;
    let painter = ui.painter().with_clip_rect(band);

    if text_width <= band.width() {
        painter.galley(egui::pos2(band.center().x - text_width / 2.0, top), galley, color);
        return;
    }

    let gap = 48.0;
    let step = text_width + gap;
    let period = f64::from(step);
    #[expect(clippy::cast_possible_truncation, reason = "décalage de défilement exprimé en pixels")]
    let offset = (ui.input(|input| input.time) * 36.0 % period) as f32;
    let mut x = band.left() - offset;
    while x < band.right() {
        painter.galley(egui::pos2(x, top), galley.clone(), color);
        x += step;
    }
    ui.ctx().request_repaint_after(Duration::from_millis(33));
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
        if let Some((message, band_color, text_color)) = self.banner() {
            let band = egui::Rect::from_min_max(egui::pos2(bounds.left(), bounds.bottom() - BAND_HEIGHT), bounds.right_bottom());
            ui.painter().rect_filled(band, 0.0, band_color);
            paint_marquee(ui, band, message, text_color);
        }
    }
}
