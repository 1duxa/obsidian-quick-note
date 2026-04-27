use std::process::Command;

use chrono::Local;
use directories::ProjectDirs;
use eframe::{
    App,
    egui::{
        self, Color32, CornerRadius, FontId, Frame, KeyboardShortcut, Margin, Modifiers, RichText,
        Shadow, Stroke, TextEdit, Vec2, ViewportBuilder,
    },
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
enum NoteMode {
    NoDate,
    Date,
}

#[derive(Serialize, Deserialize)]
struct AppConfig {
    mode: NoteMode,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            mode: NoteMode::NoDate,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct DailyNote {
    note: String,
    error: Option<String>,
    mode: NoteMode,
    config: AppConfig,
    config_path: Option<std::path::PathBuf>,
}

impl DailyNote {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (config, config_path) = Self::load_config();
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "Inter".to_owned(),
            egui::FontData::from_static(include_bytes!("../Lato-Regular.ttf")).into(),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "Inter".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        let app = Self {
            note: String::new(),
            error: None,
            mode: config.mode,
            config,
            config_path,
        };

        if let Some(storage) = cc.storage
            && let Some(v) = eframe::get_value(storage, eframe::APP_KEY)
        {
            v
        } else {
            app
        }
    }

    fn load_config() -> (AppConfig, Option<std::path::PathBuf>) {
        if let Some(proj_dirs) = ProjectDirs::from("", "", "DailyNote") {
            let config_dir = proj_dirs.config_dir();
            let config_path = config_dir.join("config.ron");

            if let Ok(content) = std::fs::read_to_string(&config_path)
                && let Ok(cfg) = ron::from_str::<AppConfig>(&content)
            {
                return (cfg, Some(config_path));
            }

            let _ = std::fs::create_dir_all(config_dir);
            (AppConfig::default(), Some(config_path))
        } else {
            (AppConfig::default(), None)
        }
    }

    fn save_config(&mut self) {
        if let Some(path) = &self.config_path {
            self.config.mode = self.mode;
            if let Ok(content) = ron::ser::to_string_pretty(&self.config, Default::default()) {
                let _ = std::fs::write(path, content);
            }
        }
    }

    fn get_cmd(content: &str) -> Command {
        let mut cmd = Command::new("obsidian");
        cmd.arg("daily:append").arg(format!("content={}", content));
        cmd
    }

    fn send_note(&mut self) -> Result<(), String> {
        if self.note.trim().is_empty() {
            return Ok(());
        }

        let content = self.note.drain(..).collect::<String>().trim().to_string();

        let final_content = if self.mode == NoteMode::Date {
            let now = Local::now();
            format!("{} {}", now.format("%H:%M"), content)
        } else {
            content
        };

        Self::get_cmd(&final_content)
            .spawn()
            .map_err(|e| e.to_string())?
            .wait()
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    fn handle_note(&mut self) {
        self.error = self.send_note().err();
        if self.error.is_none() {
            self.note.clear();
        }
    }
}

const MAX_VISIBLE_LINES: usize = 2;

impl App for DailyNote {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let mut style = (*ctx.style()).clone();
        style.visuals.window_fill = Color32::from_rgb(18, 18, 24);
        style.visuals.panel_fill = Color32::from_rgb(18, 18, 24);

        style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(35, 35, 45);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(45, 45, 60);
        style.visuals.widgets.active.bg_fill = Color32::from_rgb(55, 55, 75);

        style.visuals.override_text_color = Some(Color32::from_rgb(210, 210, 220));
        style.visuals.window_corner_radius = CornerRadius::same(12);
        style.visuals.widgets.noninteractive.corner_radius = CornerRadius::same(8);
        style.visuals.widgets.inactive.corner_radius = CornerRadius::same(8);

        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(50, 50, 62));
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(80, 80, 100));

        style.spacing.item_spacing = Vec2::new(8.0, 6.0);
        style.spacing.button_padding = Vec2::new(8.0, 4.0);
        style.spacing.window_margin = Margin::same(12);

        ctx.set_style(style);

        egui::TopBottomPanel::bottom("status_bar")
            .exact_height(6.0)
            .show_separator_line(false)
            .frame(Frame {
                inner_margin: Margin::ZERO,
                fill: Color32::TRANSPARENT,
                stroke: Stroke::NONE,
                corner_radius: CornerRadius::ZERO,
                outer_margin: Margin::ZERO,
                shadow: Shadow::NONE,
            })
            .show(ctx, |ui| {
                let mode_color = match self.mode {
                    NoteMode::NoDate => Color32::from_rgb(100, 160, 255),
                    NoteMode::Date => Color32::from_rgb(255, 100, 100),
                };
                let rect = ui.max_rect();
                ui.painter().rect_filled(rect, 0.0, mode_color);
            });

        egui::CentralPanel::default()
            .frame(
                Frame::NONE
                    .fill(Color32::from_rgb(18, 18, 24))
                    .inner_margin(Margin::same(12)),
            )
            .show(ctx, |ui| {
                // Top bar with title + icon button
                ui.horizontal(|ui| {
                    ui.heading(RichText::new("Daily Note").strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                        ui.add(
                            egui::Image::from_bytes(
                                "bytes://info.png",
                                include_bytes!("../info.png").as_slice(),
                            )
                            .fit_to_exact_size(Vec2::splat(28.0))
                            .sense(egui::Sense::hover()),
                        )
                        .on_hover_text(
                            "Shortcuts:\n\
                            Enter – Send note & close\n\
                            Shift+Enter – New line\n\
                            Ctrl+Enter – Toggle date mode\n\
                            Esc – Close without saving",
                        );
                    });
                });

                ui.add_space(8.0);

                let text_edit_frame = Frame::default()
                    .fill(Color32::from_rgb(30, 30, 38))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(60, 60, 70)))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(Margin::symmetric(10, 10));

                text_edit_frame.show(ui, |ui| {
                    let edit = TextEdit::multiline(&mut self.note)
                        .hint_text("What happened today?")
                        .desired_width(f32::INFINITY)
                        .desired_rows(MAX_VISIBLE_LINES)
                        .font(FontId::proportional(15.0))
                        .frame(false)
                        .return_key(Some(KeyboardShortcut::new(
                            Modifiers::SHIFT,
                            egui::Key::Enter,
                        )))
                        .lock_focus(true);

                    let output = edit.show(ui);
                    output.response.request_focus();

                    if output.galley.rows.len() > MAX_VISIBLE_LINES {
                        let max_chars: usize = output.galley.rows[..MAX_VISIBLE_LINES]
                            .iter()
                            .map(|r| r.char_count_excluding_newline())
                            .sum();
                        let cutoff = self
                            .note
                            .char_indices()
                            .nth(max_chars)
                            .map(|(i, _)| i)
                            .unwrap_or(self.note.len());
                        self.note.truncate(cutoff);
                    }
                });
                if let Some(err) = &self.error {
                    ui.add_space(8.0);
                    ui.label(RichText::new(err).color(Color32::RED).size(13.0));
                }
            });
        if ctx.input_mut(|i| i.consume_key(Modifiers::CTRL, egui::Key::Enter)) {
            self.mode = match self.mode {
                NoteMode::NoDate => NoteMode::Date,
                NoteMode::Date => NoteMode::NoDate,
            };
        } else if ctx.input_mut(|i| {
            i.modifiers == Modifiers::NONE && i.consume_key(Modifiers::NONE, egui::Key::Enter)
        }) {
            self.handle_note();
            if self.error.is_none() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }

        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, egui::Key::Escape)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.save_config();
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
        self.save_config();
    }
}

fn main() {
    let mut options = eframe::NativeOptions::default();

    let viewport = ViewportBuilder::default()
        .with_app_id("daily-note")
        .with_taskbar(false)
        .with_maximized(false)
        .with_always_on_top()
        .with_titlebar_buttons_shown(false)
        .with_minimize_button(false)
        .with_resizable(false);

    options.centered = true;
    options.viewport = viewport;
    eframe::run_native(
        "Daily Note",
        options,
        Box::new(|cc| Ok(Box::new(DailyNote::new(cc)))),
    )
    .unwrap();
}
