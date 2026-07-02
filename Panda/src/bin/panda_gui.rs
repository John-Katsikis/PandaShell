use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Instant;

use eframe::egui;
use egui::text::{LayoutJob, TextFormat};
use portable_pty::{native_pty_system, Child, CommandBuilder, PtySize};

const SUGGESTIONS: &[&str] = &[
    "doctor",
    "watch --once",
    "weather Athens --tomorrow",
    "gitinfo",
    "tree",
    "json file panda-demo.json",
    "qr --compact \"https://example.com\"",
    "spark --compact --compare hello panda",
    "ollama run llama3.2",
];

const CONTENT_WIDTH: f32 = 860.0;
const OUTPUT_FONT_SIZE: f32 = 12.5;
const PTY_WINDOW_HEIGHT: f32 = 245.0;
const OLLAMA_WINDOW_HEIGHT: f32 = 390.0;

struct TranscriptEntry {
    command: String,
    output: String,
    success: bool,
    duration: String,
    pty: Option<PtySession>,
}

struct PandaGui {
    command: String,
    transcript: Vec<TranscriptEntry>,
    history: Vec<String>,
    history_index: Option<usize>,
    status: String,
    cwd: PathBuf,
    previous_cwd: Option<PathBuf>,
    active_pty: Option<usize>,
    pty_text_input_active: bool,
}

struct PtySession {
    rx: Receiver<Vec<u8>>,
    writer: Box<dyn Write + Send>,
    child: Box<dyn Child + Send + Sync>,
    parser: vt100::Parser,
    live: bool,
    input_buffer: String,
}

impl Default for PandaGui {
    fn default() -> Self {
        Self {
            command: String::new(),
            transcript: vec![TranscriptEntry {
                command: "welcome".into(),
                output: "Panda terminal is ready. Type a command below or choose a suggestion."
                    .into(),
                success: true,
                duration: "now".into(),
                pty: None,
            }],
            history: Vec::new(),
            history_index: None,
            status: "Ready".into(),
            cwd: std::env::current_dir().unwrap_or_else(|_| home_dir()),
            previous_cwd: None,
            active_pty: None,
            pty_text_input_active: false,
        }
    }
}

impl eframe::App for PandaGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        apply_theme(ctx);
        self.refresh_pty_sessions(ctx);
        self.pty_text_input_active = false;

        egui::TopBottomPanel::top("title_bar")
            .frame(egui::Frame::default().fill(egui::Color32::from_rgb(11, 14, 20)))
            .show(ctx, |ui| {
                centered_content(ui, |ui| {
                    ui.add_space(9.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Panda")
                                .size(22.0)
                                .strong()
                                .color(egui::Color32::from_rgb(127, 255, 170)),
                        );
                        ui.label(
                            egui::RichText::new("workspace")
                                .size(14.0)
                                .color(egui::Color32::from_rgb(113, 210, 255)),
                        );
                        ui.add_space(10.0);
                        status_pill(
                            ui,
                            "live",
                            &self.live_pty_count().to_string(),
                            egui::Color32::from_rgb(113, 210, 255),
                        );
                        status_pill(
                            ui,
                            "cwd",
                            &short_path(&self.cwd),
                            egui::Color32::from_rgb(127, 255, 170),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new(&self.status).weak());
                        });
                    });
                    ui.add_space(9.0);
                });
            });

        egui::TopBottomPanel::bottom("prompt")
            .frame(egui::Frame::default().fill(egui::Color32::from_rgb(9, 12, 17)))
            .show(ctx, |ui| {
                centered_content(ui, |ui| {
                    ui.add_space(8.0);
                    self.suggestion_row(ui);
                    ui.add_space(6.0);
                    self.prompt_row(ui);
                    ui.add_space(9.0);
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(egui::Color32::from_rgb(8, 10, 14)))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        centered_content(ui, |ui| {
                            self.workspace(ui);
                        });
                    });
            });

        self.forward_keyboard_to_active_pty(ctx);
    }
}

impl PandaGui {
    fn workspace(&mut self, ui: &mut egui::Ui) {
        ui.add_space(14.0);

        let mut index = 0usize;
        while index < self.transcript.len() {
            if is_live_pty(&self.transcript[index]) {
                let mut live_run = Vec::new();
                while index < self.transcript.len() && is_live_pty(&self.transcript[index]) {
                    live_run.push(index);
                    index += 1;
                }
                self.live_window_grid(ui, &live_run);
            } else {
                let is_active = self.active_pty == Some(index);
                if draw_entry(
                    ui,
                    &mut self.transcript[index],
                    is_active,
                    EntryMode::History,
                    &mut self.pty_text_input_active,
                ) {
                    self.focus_pty(index);
                }
                ui.add_space(9.0);
                index += 1;
            }
        }
    }

    fn live_window_grid(&mut self, ui: &mut egui::Ui, indices: &[usize]) {
        let available = ui.available_width();
        let columns = if available >= 720.0 { 2 } else { 1 };
        let gap = 10.0;

        let mut cursor = 0usize;
        while cursor < indices.len() {
            let index = indices[cursor];
            if is_ollama_entry(&self.transcript[index]) {
                self.draw_window_slot(ui, index, available, ollama_window_height());
                ui.add_space(gap);
                cursor += 1;
                continue;
            }

            let mut row = Vec::new();
            while cursor < indices.len()
                && row.len() < columns
                && !is_ollama_entry(&self.transcript[indices[cursor]])
            {
                row.push(indices[cursor]);
                cursor += 1;
            }

            let pane_width = if row.len() == 1 {
                available
            } else {
                (available - gap) / row.len() as f32
            };

            ui.horizontal(|ui| {
                for (column, index) in row.iter().enumerate() {
                    self.draw_window_slot(ui, *index, pane_width, PTY_WINDOW_HEIGHT);

                    if column + 1 < row.len() {
                        ui.add_space(gap);
                    }
                }
            });
            ui.add_space(gap);
        }
    }

    fn draw_window_slot(&mut self, ui: &mut egui::Ui, index: usize, width: f32, height: f32) {
        let is_active = self.active_pty == Some(index);
        let response = ui.allocate_ui_with_layout(
            egui::vec2(width, height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                draw_entry(
                    ui,
                    &mut self.transcript[index],
                    is_active,
                    EntryMode::Window,
                    &mut self.pty_text_input_active,
                )
            },
        );

        if response.inner {
            self.focus_pty(index);
        }
    }

    fn live_pty_count(&self) -> usize {
        self.transcript
            .iter()
            .filter(|entry| is_live_pty(entry))
            .count()
    }

    fn focus_pty(&mut self, index: usize) {
        self.active_pty = Some(index);
        self.status = format!("Window focused: `{}`", self.transcript[index].command);
    }

    fn suggestion_row(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.label(egui::RichText::new("try").weak());
            for suggestion in SUGGESTIONS {
                if ui
                    .add(
                        egui::Button::new(egui::RichText::new(*suggestion).monospace().size(12.0))
                            .small(),
                    )
                    .clicked()
                {
                    self.command = (*suggestion).to_string();
                }
            }

            if ui.button("clear").clicked() {
                self.transcript.clear();
                self.status = "Transcript cleared".into();
            }
        });
    }

    fn prompt_row(&mut self, ui: &mut egui::Ui) {
        egui::Frame::default()
            .fill(egui::Color32::from_rgb(15, 19, 27))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(47, 60, 76)))
            .corner_radius(egui::CornerRadius::same(7))
            .inner_margin(egui::Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("[{}] >", short_path(&self.cwd)))
                            .monospace()
                            .strong()
                            .color(egui::Color32::from_rgb(127, 255, 170)),
                    );

                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.command)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .hint_text("doctor"),
                    );

                    if response.has_focus() {
                        self.active_pty = None;
                        if ui.input(|input| input.key_pressed(egui::Key::ArrowUp)) {
                            self.recall_history(-1);
                        }
                        if ui.input(|input| input.key_pressed(egui::Key::ArrowDown)) {
                            self.recall_history(1);
                        }
                    }

                    let enter_pressed = response.lost_focus()
                        && ui.input(|input| input.key_pressed(egui::Key::Enter));

                    if ui.button("Run").clicked() || enter_pressed {
                        self.run_current();
                        response.request_focus();
                    }
                });
            });
    }

    fn recall_history(&mut self, direction: isize) {
        if self.history.is_empty() {
            return;
        }

        let current = self.history_index.unwrap_or(self.history.len());
        let next = if direction < 0 {
            current.saturating_sub(1)
        } else {
            (current + 1).min(self.history.len())
        };

        self.history_index = (next < self.history.len()).then_some(next);
        self.command = self
            .history_index
            .map(|index| self.history[index].clone())
            .unwrap_or_default();
    }

    fn run_current(&mut self) {
        let command = self.command.trim().to_string();
        if command.is_empty() {
            self.status = "Type a command first".into();
            return;
        }

        self.status = format!("Running `{command}`");
        let start = Instant::now();
        let result = self.run_command(&command);
        let duration = format!("{:.2}s", start.elapsed().as_secs_f32());

        match result {
            CommandResult {
                output,
                success: true,
                pty,
            } => {
                self.status = format!("Finished in {duration}");
                if pty.is_some() {
                    self.status =
                        "PTY started in its own block. Click it when you want to type into it."
                            .into();
                }
                self.push_entry(command, output, true, duration, pty);
            }
            CommandResult {
                output,
                success: false,
                pty,
            } => {
                self.status = format!("Failed in {duration}");
                self.push_entry(command, output, false, duration, pty);
            }
        }

        self.command.clear();
        self.history_index = None;
    }

    fn run_command(&mut self, command: &str) -> CommandResult {
        match parse_local_command(command) {
            LocalCommand::Cd(target) => self.change_dir(target),
            LocalCommand::Pwd => CommandResult {
                output: format!("{}\n", self.cwd.display()),
                success: true,
                pty: None,
            },
            LocalCommand::Pty(reason) => launch_pty_command(command, &self.cwd, &reason),
            LocalCommand::External => run_panda_command(command, &self.cwd),
        }
    }

    fn change_dir(&mut self, target: Option<String>) -> CommandResult {
        let next = match target.as_deref() {
            Some("-") => match &self.previous_cwd {
                Some(path) => path.clone(),
                None => {
                    return CommandResult {
                        output: "cd: no previous directory\n".into(),
                        success: false,
                        pty: None,
                    };
                }
            },
            Some(path) => expand_path(path, &self.cwd),
            None => home_dir(),
        };

        let canonical = match std::fs::canonicalize(&next) {
            Ok(path) if path.is_dir() => path,
            Ok(_) => {
                return CommandResult {
                    output: format!("cd: not a directory: {}\n", next.display()),
                    success: false,
                    pty: None,
                };
            }
            Err(e) => {
                return CommandResult {
                    output: format!("cd: {}: {e}\n", next.display()),
                    success: false,
                    pty: None,
                };
            }
        };

        self.previous_cwd = Some(self.cwd.clone());
        self.cwd = canonical;
        CommandResult {
            output: format!("{}\n", self.cwd.display()),
            success: true,
            pty: None,
        }
    }

    fn push_entry(
        &mut self,
        command: String,
        output: String,
        success: bool,
        duration: String,
        pty: Option<PtySession>,
    ) {
        if self.history.last() != Some(&command) {
            self.history.push(command.clone());
        }

        let is_pty = pty.is_some();
        self.transcript.push(TranscriptEntry {
            command,
            output,
            success,
            duration,
            pty,
        });

        if is_pty {
            self.active_pty = None;
        }
    }

    fn refresh_pty_sessions(&mut self, ctx: &egui::Context) {
        let mut changed = false;

        for entry in &mut self.transcript {
            let Some(session) = entry.pty.as_mut() else {
                continue;
            };

            for bytes in session.rx.try_iter() {
                session.parser.process(&bytes);
                changed = true;
            }

            if session.live {
                match session.child.try_wait() {
                    Ok(Some(status)) => {
                        session.live = false;
                        entry.success = status.success();
                        entry.duration = if status.success() {
                            "done".into()
                        } else {
                            "exited".into()
                        };
                        changed = true;
                    }
                    Ok(None) => {}
                    Err(_) => {
                        session.live = false;
                        entry.success = false;
                        entry.duration = "ended".into();
                        changed = true;
                    }
                }
            }
        }

        if changed {
            ctx.request_repaint();
        }
    }

    fn forward_keyboard_to_active_pty(&mut self, ctx: &egui::Context) {
        let Some(index) = self.active_pty else {
            return;
        };
        if self.pty_text_input_active {
            return;
        }
        let Some(entry) = self.transcript.get_mut(index) else {
            self.active_pty = None;
            return;
        };
        let Some(session) = entry.pty.as_mut() else {
            self.active_pty = None;
            return;
        };
        if !session.live {
            return;
        }

        let application_cursor = session.parser.screen().application_cursor();
        let bytes = ctx.input(|input| pty_input_bytes(input, application_cursor));
        if bytes.is_empty() {
            return;
        }

        if session.writer.write_all(&bytes).is_ok() {
            let _ = session.writer.flush();
            ctx.request_repaint();
        }
    }
}

#[derive(Clone, Copy)]
enum EntryMode {
    Window,
    History,
}

fn draw_entry(
    ui: &mut egui::Ui,
    entry: &mut TranscriptEntry,
    is_active: bool,
    mode: EntryMode,
    pty_text_input_active: &mut bool,
) -> bool {
    let is_window = matches!(mode, EntryMode::Window);
    let is_ollama = is_ollama_entry(entry);
    let stroke_color = if is_active {
        egui::Color32::from_rgb(113, 210, 255)
    } else if entry.success {
        egui::Color32::from_rgb(55, 102, 83)
    } else {
        egui::Color32::from_rgb(154, 64, 72)
    };

    let mut clicked = false;

    let frame_response = egui::Frame::default()
        .fill(if is_window {
            egui::Color32::from_rgb(13, 17, 24)
        } else {
            egui::Color32::from_rgb(14, 18, 25)
        })
        .stroke(egui::Stroke::new(
            if is_active { 1.6 } else { 1.0 },
            stroke_color,
        ))
        .corner_radius(egui::CornerRadius::same(7))
        .inner_margin(egui::Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let dot_color = if is_active {
                    egui::Color32::from_rgb(113, 210, 255)
                } else if entry.pty.as_ref().is_some_and(|session| session.live) {
                    egui::Color32::from_rgb(127, 255, 170)
                } else if entry.success {
                    egui::Color32::from_rgb(98, 178, 130)
                } else {
                    egui::Color32::from_rgb(255, 105, 115)
                };
                ui.colored_label(dot_color, "●");
                ui.label(
                    egui::RichText::new(if is_ollama {
                        "ollama"
                    } else if is_window {
                        "window"
                    } else {
                        "[PANDA] >"
                    })
                    .monospace()
                    .size(12.0)
                    .strong()
                    .color(egui::Color32::from_rgb(127, 255, 170)),
                );
                ui.label(
                    egui::RichText::new(&entry.command)
                        .monospace()
                        .size(12.0)
                        .color(egui::Color32::from_rgb(230, 235, 245)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(session) = entry.pty.as_mut() {
                        let stop_fill = egui::Color32::from_rgb(156, 42, 50);
                        let stop_button = egui::Button::new(
                            egui::RichText::new("Stop")
                                .strong()
                                .color(text_on_fill(stop_fill)),
                        )
                        .fill(stop_fill)
                        .stroke(egui::Stroke::new(
                            1.0,
                            egui::Color32::from_rgb(255, 105, 115),
                        ));

                        if session.live && ui.add(stop_button).clicked() {
                            let _ = session.child.kill();
                            session.live = false;
                            entry.success = false;
                            entry.duration = "stopped".into();
                        }
                    }

                    let live = entry.pty.as_ref().is_some_and(|session| session.live);
                    let duration = if live {
                        "live"
                    } else {
                        entry.duration.as_str()
                    };
                    if is_active {
                        badge(ui, "focused", egui::Color32::from_rgb(113, 210, 255));
                    }
                    if is_ollama {
                        badge(ui, "chat", egui::Color32::from_rgb(235, 126, 255));
                    }
                    badge(
                        ui,
                        duration,
                        if live {
                            egui::Color32::from_rgb(127, 255, 170)
                        } else {
                            egui::Color32::from_rgb(145, 154, 170)
                        },
                    );
                });
            });

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(3.0);

            let fallback_color = if entry.success {
                egui::Color32::from_rgb(218, 224, 235)
            } else {
                egui::Color32::from_rgb(255, 190, 190)
            };

            let output = if let Some(session) = entry.pty.as_ref() {
                let contents = session.parser.screen().contents();
                if contents.trim().is_empty() {
                    "(PTY is running)".to_string()
                } else {
                    contents
                }
            } else if entry.output.trim().is_empty() {
                "(no output)".to_string()
            } else {
                entry.output.clone()
            };

            if is_window || entry.pty.is_some() {
                let max_height = if is_window && is_ollama {
                    ollama_window_height() - 118.0
                } else if is_window {
                    PTY_WINDOW_HEIGHT - 70.0
                } else {
                    360.0
                };
                egui::ScrollArea::vertical()
                    .max_height(max_height)
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let response = ui.add(
                            egui::Label::new(ansi_layout_job(&output, fallback_color))
                                .wrap()
                                .sense(egui::Sense::click()),
                        );
                        clicked = response.clicked();
                    });
            } else {
                let response = ui.add(
                    egui::Label::new(ansi_layout_job(&output, fallback_color))
                        .wrap()
                        .sense(egui::Sense::click()),
                );
                clicked = response.clicked();
            }

            if is_window && is_ollama {
                ui.add_space(6.0);
                draw_ollama_input(ui, entry, pty_text_input_active);
            }
        });

    entry.pty.as_ref().is_some_and(|session| session.live)
        && (clicked || frame_response.response.clicked())
}

fn is_live_pty(entry: &TranscriptEntry) -> bool {
    entry.pty.as_ref().is_some_and(|session| session.live)
}

fn is_ollama_entry(entry: &TranscriptEntry) -> bool {
    panda::parser::parse_line(&entry.command)
        .ok()
        .and_then(|ast| ast.commands.first().map(|command| command.name == "ollama"))
        .unwrap_or(false)
}

fn ollama_window_height() -> f32 {
    OLLAMA_WINDOW_HEIGHT
}

fn draw_ollama_input(
    ui: &mut egui::Ui,
    entry: &mut TranscriptEntry,
    pty_text_input_active: &mut bool,
) {
    let Some(session) = entry.pty.as_mut() else {
        return;
    };
    if !session.live {
        return;
    }

    ui.horizontal(|ui| {
        let response = ui.add(
            egui::TextEdit::singleline(&mut session.input_buffer)
                .font(egui::TextStyle::Monospace)
                .desired_width(f32::INFINITY)
                .hint_text("Ask Ollama..."),
        );
        let enter_pressed =
            response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
        if response.has_focus() || enter_pressed {
            *pty_text_input_active = true;
        }

        let break_fill = egui::Color32::from_rgb(218, 139, 72);
        let break_button = egui::Button::new(
            egui::RichText::new("Break")
                .strong()
                .color(text_on_fill(break_fill)),
        )
        .fill(break_fill)
        .stroke(egui::Stroke::new(
            1.0,
            egui::Color32::from_rgb(255, 190, 120),
        ));
        if ui.add(break_button).clicked() {
            let _ = session.writer.write_all(&[0x03]);
            let _ = session.writer.flush();
        }

        let ask_fill = egui::Color32::from_rgb(127, 255, 170);
        let send = egui::Button::new(
            egui::RichText::new("Ask")
                .strong()
                .color(text_on_fill(ask_fill)),
        )
        .fill(ask_fill)
        .stroke(egui::Stroke::new(
            1.0,
            egui::Color32::from_rgb(176, 255, 205),
        ));

        if (ui.add(send).clicked() || enter_pressed) && !session.input_buffer.trim().is_empty() {
            let mut message = std::mem::take(&mut session.input_buffer);
            message.push('\r');
            let _ = session.writer.write_all(message.as_bytes());
            let _ = session.writer.flush();
        }
    });
}

fn badge(ui: &mut egui::Ui, text: &str, color: egui::Color32) {
    let bright = color_luminance(color) > 0.55;
    let fill = if bright {
        color
    } else {
        egui::Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 42)
    };
    let text_color = if bright { text_on_fill(color) } else { color };
    let stroke_color = if bright {
        egui::Color32::from_rgba_premultiplied(
            (color.r() as f32 * 0.72) as u8,
            (color.g() as f32 * 0.72) as u8,
            (color.b() as f32 * 0.72) as u8,
            210,
        )
    } else {
        egui::Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 95)
    };

    egui::Frame::default()
        .fill(fill)
        .stroke(egui::Stroke::new(1.0, stroke_color))
        .corner_radius(egui::CornerRadius::same(5))
        .inner_margin(egui::Margin::symmetric(6, 2))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(text)
                    .monospace()
                    .size(10.5)
                    .color(text_color),
            );
        });
}

fn status_pill(ui: &mut egui::Ui, label: &str, value: &str, color: egui::Color32) {
    egui::Frame::default()
        .fill(egui::Color32::from_rgb(15, 20, 29))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(38, 49, 64)))
        .corner_radius(egui::CornerRadius::same(6))
        .inner_margin(egui::Margin::symmetric(7, 3))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(label)
                        .size(10.5)
                        .color(egui::Color32::from_rgb(126, 137, 154)),
                );
                ui.label(
                    egui::RichText::new(value)
                        .monospace()
                        .size(10.5)
                        .color(color),
                );
            });
        });
}

fn text_on_fill(fill: egui::Color32) -> egui::Color32 {
    if color_luminance(fill) > 0.55 {
        egui::Color32::from_rgb(8, 10, 14)
    } else {
        egui::Color32::from_rgb(255, 245, 245)
    }
}

fn color_luminance(color: egui::Color32) -> f32 {
    (0.299 * color.r() as f32 + 0.587 * color.g() as f32 + 0.114 * color.b() as f32) / 255.0
}

fn centered_content(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    let available_width = ui.available_width();
    let content_width = available_width.min(CONTENT_WIDTH);
    let margin = ((available_width - content_width) / 2.0).max(0.0);

    ui.horizontal(|ui| {
        ui.add_space(margin);
        ui.allocate_ui_with_layout(
            egui::vec2(content_width, 0.0),
            egui::Layout::top_down(egui::Align::Min),
            add_contents,
        );
    });
}

struct CommandResult {
    output: String,
    success: bool,
    pty: Option<PtySession>,
}

enum LocalCommand {
    Cd(Option<String>),
    Pwd,
    Pty(String),
    External,
}

fn parse_local_command(command: &str) -> LocalCommand {
    let Ok(ast) = panda::parser::parse_line(command) else {
        return LocalCommand::External;
    };
    let Some(command) = ast.commands.first() else {
        return LocalCommand::External;
    };

    match command.name.as_str() {
        "cd" => LocalCommand::Cd(command.args.first().cloned()),
        "pwd" => LocalCommand::Pwd,
        "serve" => LocalCommand::Pty("serve is running inside Panda's embedded PTY".into()),
        "watch"
            if command
                .args
                .iter()
                .any(|arg| arg == "--live" || arg == "-l") =>
        {
            LocalCommand::Pty("watch --live is running inside Panda's embedded PTY".into())
        }
        "timer" if command.args.first().is_some_and(|arg| arg == "stopwatch") => {
            LocalCommand::Pty("timer stopwatch is running inside Panda's embedded PTY".into())
        }
        name if needs_real_terminal(name) => {
            LocalCommand::Pty(format!("{name} is running inside Panda's embedded PTY"))
        }
        _ => LocalCommand::External,
    }
}

fn needs_real_terminal(command: &str) -> bool {
    matches!(
        command,
        "nano"
            | "vim"
            | "vi"
            | "nvim"
            | "emacs"
            | "less"
            | "more"
            | "man"
            | "top"
            | "htop"
            | "ssh"
            | "ftp"
            | "sftp"
            | "python"
            | "python3"
            | "irb"
            | "node"
            | "ollama"
    )
}

fn launch_pty_command(command: &str, cwd: &Path, reason: &str) -> CommandResult {
    match create_pty_session(command, cwd) {
        Ok(pty) => CommandResult {
            output: format!("{reason}.\nClick this pane to type into the running command.\n"),
            success: true,
            pty: Some(pty),
        },
        Err(e) => CommandResult {
            output: format!("Failed to start embedded PTY for `{command}`: {e}\n"),
            success: false,
            pty: None,
        },
    }
}

fn create_pty_session(command: &str, cwd: &Path) -> Result<PtySession, String> {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 26,
            cols: 96,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| e.to_string())?;

    let mut cmd = CommandBuilder::new(panda_binary());
    cmd.args(["--run", command]);
    cmd.cwd(cwd);
    cmd.env("PATH", gui_command_path());
    cmd.env("LANG", "en_US.UTF-8");
    cmd.env("LC_ALL", "en_US.UTF-8");
    cmd.env("LC_CTYPE", "UTF-8");
    cmd.env("TERM", "xterm-256color");

    let child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| format!("spawn failed: {e}"))?;
    let mut reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| format!("reader failed: {e}"))?;
    let writer = pair
        .master
        .take_writer()
        .map_err(|e| format!("writer failed: {e}"))?;

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut buffer = [0u8; 8192];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(count) => {
                    if tx.send(buffer[..count].to_vec()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    Ok(PtySession {
        rx,
        writer,
        child,
        parser: vt100::Parser::new(26, 96, 400),
        live: true,
        input_buffer: String::new(),
    })
}

fn run_panda_command(command: &str, cwd: &Path) -> CommandResult {
    let panda = panda_binary();
    let output = Command::new(&panda)
        .args(["--run", command])
        .current_dir(cwd)
        .env("PATH", gui_command_path())
        .env("LANG", "en_US.UTF-8")
        .env("LC_ALL", "en_US.UTF-8")
        .env("LC_CTYPE", "UTF-8")
        .output();

    match output {
        Ok(output) => {
            let mut text = String::new();
            text.push_str(&String::from_utf8_lossy(&output.stdout));
            text.push_str(&String::from_utf8_lossy(&output.stderr));
            CommandResult {
                output: text,
                success: output.status.success(),
                pty: None,
            }
        }
        Err(e) => CommandResult {
            output: format!("Failed to run {}: {e}", panda.display()),
            success: false,
            pty: None,
        },
    }
}

fn gui_command_path() -> String {
    let current = std::env::var("PATH").unwrap_or_default();
    let defaults = [
        "/opt/homebrew/bin",
        "/usr/local/bin",
        "/usr/bin",
        "/bin",
        "/usr/sbin",
        "/sbin",
    ];
    let mut parts = defaults
        .iter()
        .map(|path| (*path).to_string())
        .collect::<Vec<_>>();

    for path in current.split(':').filter(|path| !path.is_empty()) {
        if !parts.iter().any(|existing| existing == path) {
            parts.push(path.to_string());
        }
    }

    parts.join(":")
}

fn pty_input_bytes(input: &egui::InputState, application_cursor: bool) -> Vec<u8> {
    let mut bytes = Vec::new();

    for event in &input.events {
        match event {
            egui::Event::Text(text) => bytes.extend_from_slice(text.as_bytes()),
            egui::Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } => {
                if modifiers.ctrl {
                    if let Some(byte) = ctrl_key_byte(*key) {
                        bytes.push(byte);
                    }
                    continue;
                }

                match key {
                    egui::Key::Enter => bytes.push(b'\r'),
                    egui::Key::Backspace => bytes.push(0x7f),
                    egui::Key::Tab => bytes.push(b'\t'),
                    egui::Key::Escape => bytes.push(0x1b),
                    egui::Key::ArrowUp => {
                        bytes.extend_from_slice(cursor_key(b'A', application_cursor))
                    }
                    egui::Key::ArrowDown => {
                        bytes.extend_from_slice(cursor_key(b'B', application_cursor))
                    }
                    egui::Key::ArrowRight => {
                        bytes.extend_from_slice(cursor_key(b'C', application_cursor))
                    }
                    egui::Key::ArrowLeft => {
                        bytes.extend_from_slice(cursor_key(b'D', application_cursor))
                    }
                    egui::Key::Home => bytes.extend_from_slice(b"\x1b[H"),
                    egui::Key::End => bytes.extend_from_slice(b"\x1b[F"),
                    egui::Key::Delete => bytes.extend_from_slice(b"\x1b[3~"),
                    egui::Key::PageUp => bytes.extend_from_slice(b"\x1b[5~"),
                    egui::Key::PageDown => bytes.extend_from_slice(b"\x1b[6~"),
                    _ => {}
                }
            }
            _ => {}
        }
    }

    bytes
}

fn cursor_key(suffix: u8, application_cursor: bool) -> &'static [u8] {
    match (application_cursor, suffix) {
        (true, b'A') => b"\x1bOA",
        (true, b'B') => b"\x1bOB",
        (true, b'C') => b"\x1bOC",
        (true, b'D') => b"\x1bOD",
        (false, b'A') => b"\x1b[A",
        (false, b'B') => b"\x1b[B",
        (false, b'C') => b"\x1b[C",
        (false, b'D') => b"\x1b[D",
        _ => b"",
    }
}

fn ctrl_key_byte(key: egui::Key) -> Option<u8> {
    Some(match key {
        egui::Key::A => 0x01,
        egui::Key::B => 0x02,
        egui::Key::C => 0x03,
        egui::Key::D => 0x04,
        egui::Key::E => 0x05,
        egui::Key::F => 0x06,
        egui::Key::G => 0x07,
        egui::Key::H => 0x08,
        egui::Key::I => 0x09,
        egui::Key::J => 0x0a,
        egui::Key::K => 0x0b,
        egui::Key::L => 0x0c,
        egui::Key::M => 0x0d,
        egui::Key::N => 0x0e,
        egui::Key::O => 0x0f,
        egui::Key::P => 0x10,
        egui::Key::Q => 0x11,
        egui::Key::R => 0x12,
        egui::Key::S => 0x13,
        egui::Key::T => 0x14,
        egui::Key::U => 0x15,
        egui::Key::V => 0x16,
        egui::Key::W => 0x17,
        egui::Key::X => 0x18,
        egui::Key::Y => 0x19,
        egui::Key::Z => 0x1a,
        _ => return None,
    })
}

fn expand_path(path: &str, cwd: &Path) -> PathBuf {
    if path == "~" {
        return home_dir();
    }

    if let Some(rest) = path.strip_prefix("~/") {
        return home_dir().join(rest);
    }

    let path = PathBuf::from(path);
    if path.is_absolute() {
        path
    } else {
        cwd.join(path)
    }
}

fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

fn short_path(path: &Path) -> String {
    let home = home_dir();
    if let Ok(rest) = path.strip_prefix(&home) {
        if rest.as_os_str().is_empty() {
            return "~".into();
        }
        return format!("~/{}", rest.display());
    }

    path.display().to_string()
}

fn panda_binary() -> PathBuf {
    let Ok(current) = std::env::current_exe() else {
        return PathBuf::from("Panda");
    };

    if let Some(dir) = current.parent() {
        let sibling = dir.join("Panda");
        if sibling.exists() {
            return sibling;
        }
    }

    PathBuf::from("Panda")
}

#[cfg(test)]
fn strip_ansi(input: &str) -> String {
    let mut output = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            let _ = chars.next();
            for code in chars.by_ref() {
                if code.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            output.push(ch);
        }
    }

    output
}

fn ansi_layout_job(input: &str, fallback_color: egui::Color32) -> LayoutJob {
    let mut job = LayoutJob::default();
    let mut color = fallback_color;
    let mut strong = false;
    let mut dim = false;
    let mut chars = input.chars().peekable();
    let mut text = String::new();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            flush_span(&mut job, &mut text, color, strong, dim);
            let _ = chars.next();
            let mut code = String::new();
            for next in chars.by_ref() {
                if next.is_ascii_alphabetic() {
                    break;
                }
                code.push(next);
            }

            apply_ansi_code(&code, fallback_color, &mut color, &mut strong, &mut dim);
        } else {
            text.push(ch);
        }
    }

    flush_span(&mut job, &mut text, color, strong, dim);
    job
}

fn flush_span(
    job: &mut LayoutJob,
    text: &mut String,
    color: egui::Color32,
    strong: bool,
    dim: bool,
) {
    if text.is_empty() {
        return;
    }

    let mut format = TextFormat {
        font_id: egui::FontId::monospace(OUTPUT_FONT_SIZE),
        color: if dim { dim_color(color) } else { color },
        ..Default::default()
    };

    if strong {
        format.font_id = egui::FontId::monospace(OUTPUT_FONT_SIZE + 0.4);
    }

    job.append(text, 0.0, format);
    text.clear();
}

fn apply_ansi_code(
    code: &str,
    fallback_color: egui::Color32,
    color: &mut egui::Color32,
    strong: &mut bool,
    dim: &mut bool,
) {
    let parts = code
        .split(';')
        .filter_map(|part| part.parse::<u16>().ok())
        .collect::<Vec<_>>();

    if parts.is_empty() {
        return;
    }

    let mut i = 0usize;
    while i < parts.len() {
        match parts[i] {
            0 => {
                *color = fallback_color;
                *strong = false;
                *dim = false;
            }
            1 => *strong = true,
            2 => *dim = true,
            22 => {
                *strong = false;
                *dim = false;
            }
            30..=37 | 90..=97 => *color = ansi_basic_color(parts[i]),
            38 if parts.get(i + 1) == Some(&5) => {
                if let Some(value) = parts.get(i + 2) {
                    *color = ansi_256_color(*value as u8);
                    i += 2;
                }
            }
            _ => {}
        }
        i += 1;
    }
}

fn ansi_basic_color(code: u16) -> egui::Color32 {
    match code {
        30 => egui::Color32::from_rgb(35, 38, 46),
        31 => egui::Color32::from_rgb(255, 98, 106),
        32 => egui::Color32::from_rgb(92, 245, 142),
        33 => egui::Color32::from_rgb(255, 218, 94),
        34 => egui::Color32::from_rgb(116, 170, 255),
        35 => egui::Color32::from_rgb(235, 126, 255),
        36 => egui::Color32::from_rgb(103, 231, 255),
        37 => egui::Color32::from_rgb(230, 235, 245),
        90 => egui::Color32::from_rgb(115, 122, 138),
        91 => egui::Color32::from_rgb(255, 118, 126),
        92 => egui::Color32::from_rgb(127, 255, 170),
        93 => egui::Color32::from_rgb(255, 228, 112),
        94 => egui::Color32::from_rgb(139, 188, 255),
        95 => egui::Color32::from_rgb(245, 150, 255),
        96 => egui::Color32::from_rgb(132, 239, 255),
        97 => egui::Color32::WHITE,
        _ => egui::Color32::WHITE,
    }
}

fn ansi_256_color(value: u8) -> egui::Color32 {
    match value {
        51 => egui::Color32::from_rgb(0, 255, 255),
        75 => egui::Color32::from_rgb(95, 175, 255),
        82 => egui::Color32::from_rgb(95, 255, 0),
        118 => egui::Color32::from_rgb(135, 255, 0),
        141 => egui::Color32::from_rgb(175, 135, 255),
        203 => egui::Color32::from_rgb(255, 95, 95),
        213 => egui::Color32::from_rgb(255, 135, 255),
        220 => egui::Color32::from_rgb(255, 215, 0),
        0..=15 => ansi_basic_color(match value {
            0..=7 => 30 + value as u16,
            _ => 90 + (value as u16 - 8),
        }),
        16..=231 => {
            let idx = value - 16;
            let r = idx / 36;
            let g = (idx % 36) / 6;
            let b = idx % 6;
            egui::Color32::from_rgb(color_cube(r), color_cube(g), color_cube(b))
        }
        232..=255 => {
            let gray = 8 + (value - 232) * 10;
            egui::Color32::from_gray(gray)
        }
    }
}

fn color_cube(value: u8) -> u8 {
    if value == 0 {
        0
    } else {
        55 + value * 40
    }
}

fn dim_color(color: egui::Color32) -> egui::Color32 {
    egui::Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), 150)
}

fn apply_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = egui::Color32::from_rgb(8, 10, 14);
    visuals.panel_fill = egui::Color32::from_rgb(8, 10, 14);
    visuals.extreme_bg_color = egui::Color32::from_rgb(5, 7, 10);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(25, 31, 42);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(35, 45, 58);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(47, 68, 84);
    visuals.selection.bg_fill = egui::Color32::from_rgb(42, 130, 96);
    visuals.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 50, 64));
    ctx.set_visuals(visuals);
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1080.0, 760.0])
            .with_min_inner_size([760.0, 520.0])
            .with_icon(panda_window_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "Panda Terminal",
        options,
        Box::new(|_cc| Ok(Box::<PandaGui>::default())),
    )
}

fn panda_window_icon() -> egui::IconData {
    let size = 64usize;
    let mut rgba = vec![0u8; size * size * 4];

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - 31.5;
            let dy = y as f32 - 31.5;
            let mut color = None;

            if circle(dx, dy, 29.0) {
                color = Some([13, 17, 24, 255]);
            }
            if circle(dx + 19.0, dy + 18.0, 9.0) || circle(dx - 19.0, dy + 18.0, 9.0) {
                color = Some([8, 10, 14, 255]);
            }
            if circle(dx, dy, 23.0) {
                color = Some([235, 245, 238, 255]);
            }
            if circle(dx + 10.0, dy + 3.0, 6.8) || circle(dx - 10.0, dy + 3.0, 6.8) {
                color = Some([18, 24, 31, 255]);
            }
            if circle(dx + 9.0, dy + 4.0, 2.3) || circle(dx - 9.0, dy + 4.0, 2.3) {
                color = Some([127, 255, 170, 255]);
            }
            if circle(dx, dy - 6.5, 4.3) {
                color = Some([18, 24, 31, 255]);
            }
            if dy > 10.0 && dy < 13.0 && dx.abs() < 9.0 {
                color = Some([18, 24, 31, 255]);
            }
            if circle(dx, dy, 29.0) && (x < 5 || x > 58 || y < 5 || y > 58) {
                color = Some([127, 255, 170, 255]);
            }

            if let Some([r, g, b, a]) = color {
                let offset = (y * size + x) * 4;
                rgba[offset] = r;
                rgba[offset + 1] = g;
                rgba[offset + 2] = b;
                rgba[offset + 3] = a;
            }
        }
    }

    egui::IconData {
        rgba,
        width: size as u32,
        height: size as u32,
    }
}

fn circle(dx: f32, dy: f32, radius: f32) -> bool {
    dx * dx + dy * dy <= radius * radius
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use eframe::egui;

    use super::{ansi_layout_job, expand_path, parse_local_command, strip_ansi, LocalCommand};

    #[test]
    fn strips_terminal_color_codes() {
        assert_eq!(strip_ansi("\x1b[92mOK\x1b[0m plain"), "OK plain");
    }

    #[test]
    fn parses_ansi_into_multiple_colored_sections() {
        let job = ansi_layout_job("\x1b[92mOK\x1b[0m plain", egui::Color32::WHITE);

        assert!(job.sections.len() >= 2);
    }

    #[test]
    fn parses_gui_local_cd_and_pwd() {
        assert!(matches!(
            parse_local_command("cd ~/Desktop"),
            LocalCommand::Cd(Some(_))
        ));
        assert!(matches!(parse_local_command("pwd"), LocalCommand::Pwd));
        assert!(matches!(parse_local_command("ls"), LocalCommand::External));
        assert!(matches!(
            parse_local_command("nano Cargo.toml"),
            LocalCommand::Pty(_)
        ));
        assert!(matches!(
            parse_local_command("serve 8000"),
            LocalCommand::Pty(_)
        ));
        assert!(matches!(
            parse_local_command("ollama run llama3.2"),
            LocalCommand::Pty(_)
        ));
    }

    #[test]
    fn expands_relative_paths_against_cwd() {
        assert_eq!(
            expand_path("src", Path::new("/tmp/project")),
            PathBuf::from("/tmp/project/src")
        );
    }

    #[test]
    fn maps_keyboard_input_for_pty() {
        let mut input = egui::RawInput::default();
        input.events.push(egui::Event::Text("hi 🐼".into()));
        input.events.push(egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        });
        input.events.push(egui::Event::Key {
            key: egui::Key::X,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::CTRL,
        });
        input.events.push(egui::Event::Key {
            key: egui::Key::ArrowUp,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        });
        input.events.push(egui::Event::Key {
            key: egui::Key::ArrowDown,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        });

        let ctx = egui::Context::default();
        ctx.begin_pass(input.clone());
        let bytes = ctx.input(|input| super::pty_input_bytes(input, false));
        let _ = ctx.end_pass();

        assert_eq!(bytes, "hi 🐼\r\u{18}\u{1b}[A\u{1b}[B".as_bytes());

        ctx.begin_pass(input);
        let bytes = ctx.input(|input| super::pty_input_bytes(input, true));
        let _ = ctx.end_pass();

        assert_eq!(bytes, "hi 🐼\r\u{18}\u{1b}OA\u{1b}OB".as_bytes());
    }

    #[test]
    fn chooses_readable_text_for_bright_fills() {
        assert_eq!(
            super::text_on_fill(egui::Color32::from_rgb(127, 255, 170)),
            egui::Color32::from_rgb(8, 10, 14)
        );
        assert_eq!(
            super::text_on_fill(egui::Color32::from_rgb(156, 42, 50)),
            egui::Color32::from_rgb(255, 245, 245)
        );
    }
}
