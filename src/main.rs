use chrono::Local;
use directories::ProjectDirs;
use eframe::{
    App,
    egui::{
        self, Color32, CornerRadius, FontId, Frame, Id, KeyboardShortcut, Margin, Modifiers,
        RichText, Shadow, Stroke, TextEdit, Vec2, ViewportBuilder,
    },
};
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
enum NoteMode {
    NoTime,
    Time,
}
fn random_bye() -> &'static str {
    use rand::prelude::*;
    const BYE: &[&str] = &[
        "bye!",
        "see you!",
        "ciao!",
        "adiós!",
        "au revoir!",
        "tschüss!",
        "doei!",
        "hade!",
        "adjö!",
        "hej då!",
        "na zdravljé!",
        "dovidenia!",
        "viszontlátásra!",
        "laho",
        "do widzenia!",
        "até logo!",
        "hasta luego!",
        "arrivederci!",
        "a deus!",
        "pá!",
        "vale!",
        "na shledanou!",
        "farvel!",
        "hættir!",
        "bæ!",
        "góðan daginn!",
        "salaam!",
        "do svidaniya!",
        "poka!",
        "sbohem!",
        "na zdravje!",
        "nasvidenje!",
        "góðan dag!",
        "sæl!",
        "dozidānie!",
        "uz redzēšanos!",
        "sudie!",
        "labas!",
        "atskiriames!",
        "zài jiàn!",
        "bái bái!",
        "sayōnara!",
        "jā ne!",
        "annyeong!",
        "jal ga!",
        "an-nyeong-hi gase-yo!",
        "ma’a as-salaama!",
        "ilaa al-liqa!",
        "khuda hafiz!",
        "alwida!",
        "namaste!",
        "phir milenge!",
        "shubh ratri!",
        "alavida!",
        "adiós!",
        "selamat tinggal!",
        "sampai jumpa!",
        "chào!",
        "tạm biệt!",
        "kwa heri!",
        "hakuna matata!",
        "shikamoo!",
        "kwa herini!",
        "salam!",
        "khosh amadid!",
        "khodaa haafez!",
        "khodahafez!",
        "khodahafez!",
        "khodahafez!",
        "hamba kahle!",
        "sala kahle!",
        "e noho rā!",
        "haere rā!",
        "kia ora!",
        "mā te waiata!",
        "salaam!",
        "salaam alaykum!",
        "wa alaykum salaam!",
        "agur!",
        "adiós!",
        "do viđenja!",
        "doviđenja!",
        "na zdravje!",
        "dovizhdane!",
    ];

    let mut rng = rand::rng();
    BYE.choose(&mut rng).copied().unwrap_or("bye!")
}
#[derive(Serialize, Deserialize)]
struct AppConfig {
    mode: NoteMode,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            mode: NoteMode::NoTime,
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
    closing: bool,
    close_t: f32,
    bye: String,
}

impl DailyNote {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (config, config_path) = Self::load_config();

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
            closing: false,
            close_t: 0.,
            bye: random_bye().to_string(),
        };

        if let Some(storage) = cc.storage
            && let Some(mut v) = eframe::get_value::<DailyNote>(storage, eframe::APP_KEY)
        {
            v.bye = app.bye;
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
        let content = std::mem::take(&mut self.note);
        let content = content.trim();

        if content.is_empty() {
            return Ok(());
        }

        let content = content.to_owned();
        let final_content = if self.mode == NoteMode::Time {
            let now = Local::now();
            format!("{} {}", now.format("%H:%M"), content)
        } else {
            content
        };

        Self::get_cmd(&final_content)
            .spawn()
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    fn handle_note(&mut self) {
        self.error = self.send_note().err();
    }
}

const MAX_VISIBLE_LINES: usize = 4;

impl App for DailyNote {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        let mut style = (*ctx.global_style()).clone();
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

        ctx.set_global_style(style);

        if ctx.input_mut(|i| i.consume_key(Modifiers::CTRL, egui::Key::Enter)) {
            self.mode = match self.mode {
                NoteMode::NoTime => NoteMode::Time,
                NoteMode::Time => NoteMode::NoTime,
            };
        } else if ctx.input_mut(|i| {
            i.modifiers == Modifiers::NONE && i.consume_key(Modifiers::NONE, egui::Key::Enter)
        }) {
            self.handle_note();
            if self.error.is_none() {
                self.closing = true;
            }
        }

        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, egui::Key::Escape)) {
            self.closing = true;
        }
    }

    fn on_exit(&mut self) {
        self.save_config();
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
        self.save_config();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.closing {
            self.close_t += ui.ctx().input(|i| i.stable_dt).min(0.5);
            ui.ctx().request_repaint();

            egui::Area::new(Id::new("mode_overlay"))
                .order(egui::Order::Foreground)
                .interactable(false)
                .show(ui, |ui| {
                    let t = (self.close_t / 0.40).clamp(0.0, 1.0);

                    let alpha = egui::lerp(0.0..=200.0, t) as u8;

                    let rect = ui.content_rect();

                    ui.painter().rect_filled(
                        rect,
                        0.0,
                        Color32::from_rgba_unmultiplied(20, 20, 20, alpha),
                    );

                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        self.bye.clone(),
                        FontId::proportional(28.0),
                        Color32::from_rgba_unmultiplied(255, 255, 255, alpha),
                    );

                    if self.closing && t >= 1.0 {
                        self.closing = false;
                        self.close_t = 0.0;
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

            return;
        }
        egui::Panel::bottom("status_bar")
            .exact_size(24.0)
            .show_separator_line(false)
            .frame(Frame {
                inner_margin: Margin::ZERO,
                fill: Color32::from_rgb(18, 18, 24),
                stroke: Stroke::NONE,
                corner_radius: CornerRadius::ZERO,
                outer_margin: Margin::ZERO,
                shadow: Shadow::NONE,
            })
            .show_inside(ui, |ui| {
                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                    |ui| {
                        let mode_text = match self.mode {
                            NoteMode::NoTime => {
                                "--------------------------| mode: no time |--------------------------"
                            }
                            NoteMode::Time => {
                                "--------------------------| mode: time |--------------------------"
                            }
                        };
                        ui.label(
                            RichText::new(mode_text)
                                .size(12.0)
                                .color(Color32::from_gray(120)),
                        );
                    },
                );
            });
        egui::CentralPanel::default()
            .frame(
                Frame::NONE
                    .fill(Color32::from_rgb(18, 18, 24))
                    .inner_margin(Margin::same(12)),
            )
            .show_inside(ui, |ui| {
                ui.add_space(8.0);

                let text_edit_frame = Frame::default()
                    .fill(Color32::from_rgb(30, 30, 38))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(60, 60, 70)))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(Margin::symmetric(10, 10));

                text_edit_frame.show(ui, |ui| {
                    let edit = TextEdit::multiline(&mut self.note)
                        .hint_text(
                            RichText::new("What happened today?").color(Color32::from_gray(120)),
                        )
                        .desired_width(f32::INFINITY)
                        .desired_rows(MAX_VISIBLE_LINES)
                        .font(FontId::proportional(15.0))
                        .frame(Frame::NONE)
                        .return_key(Some(KeyboardShortcut::new(
                            Modifiers::SHIFT,
                            egui::Key::Enter,
                        )))
                        .lock_focus(true);

                    let output = edit.show(ui);

                    if ui.memory(|m| !m.has_focus(output.response.id)) {
                        output.response.request_focus();
                    }

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
                    let rect = output.response.rect;
                    egui::Area::new(Id::new("info_icon"))
                        .order(egui::Order::Foreground) // draw on top
                        .fixed_pos(rect.right_top() + egui::vec2(-15.0, -8.0))
                        .show(ui, |ui| {
                            let (rect, response) =
                                ui.allocate_exact_size(Vec2::splat(24.0), egui::Sense::hover());

                            let t = ui.ctx().animate_bool(response.id, response.hovered());
                            let alpha = egui::lerp(120.0..=180.0, t);

                            let painter = ui.painter();

                            painter.circle_filled(
                                rect.center(),
                                10.0,
                                Color32::from_gray(160).linear_multiply(alpha / 255.0),
                            );

                            painter.text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "i",
                                FontId::proportional(14.0),
                                Color32::from_white_alpha(180),
                            );

                            response.on_hover_text(
                                "Shortcuts:\n\
                                Enter – Send note & close\n\
                                Shift+Enter – New line\n\
                                Ctrl+Enter – Toggle date mode\n\
                                Esc – Close without saving",
                            );
                        });
                });
                if let Some(err) = &self.error {
                    ui.add_space(8.0);
                    ui.label(RichText::new(err).color(Color32::RED).size(13.0));
                }
            });
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
        .with_resizable(false)
        .with_inner_size(Vec2::new(300.0, 140.0));

    options.viewport = viewport;
    eframe::run_native(
        "Daily Note",
        options,
        Box::new(|cc| Ok(Box::new(DailyNote::new(cc)))),
    )
    .unwrap();
}
