//! Quick diary / journal recorder built from rustcars widgets, stored in SQLite.
//! Run: `cargo run --example diary [-- --db PATH]` (desktop only).
//!
//! * The NEW ENTRY field is always focused (unless you click into a search field);
//!   Enter saves the entry with the current time (stored UTC, shown local).
//! * Entry rows use click modes (AGENTS.md P2): click = select (shows Edit/Delete),
//!   double = edit in place (load into the field; Enter saves), triple = quote into a new entry.
//!   Shift/Ctrl+click or the right-click menu reach the same modes. Only one row is active.
//! * Delete asks for confirmation, and the last delete can be undone.
//! * Default database: the platform data dir (`directories::ProjectDirs`), e.g.
//!   `~/.local/share/rustcars-diary/diary.sqlite` on Linux.
mod store;

use rustcars::{
    Cap, LcarsFrame, LcarsTheme, Mode, Side, action_button, apply_theme, install_lcars_font,
    lcars_button, mode_list, palette, readout,
};
use std::collections::HashMap;
use store::{Entry, Store};

pub struct Diary {
    pub theme: LcarsTheme,
    pub store: Store,
    pub input: String,
    pub search: String,
    pub date: String,
    pub modes: HashMap<i64, Mode>,
    pub editing: Option<i64>,
    pub confirm_delete: Option<i64>,
    pub undo: Option<Entry>,
    pub status: String,
}

impl Diary {
    pub fn new(ctx: &egui::Context, store: Store) -> Self {
        let theme = LcarsTheme::default();
        install_lcars_font(ctx);
        apply_theme(ctx, &theme);
        Self {
            theme,
            store,
            input: String::new(),
            search: String::new(),
            date: String::new(),
            modes: HashMap::new(),
            editing: None,
            confirm_delete: None,
            undo: None,
            status: "READY".into(),
        }
    }

    fn selected(&self) -> Option<i64> {
        self.modes
            .iter()
            .find(|(_, m)| **m == Mode::Primary)
            .map(|(id, _)| *id)
    }

    fn save(&mut self) {
        let text = self.input.trim().to_owned();
        if text.is_empty() {
            return;
        }
        let r = match self.editing.take() {
            Some(id) => self
                .store
                .update(id, &text)
                .map(|_| "ENTRY UPDATED".to_owned()),
            None => self
                .store
                .add(&text, &Store::now_utc())
                .map(|e| format!("SAVED {}", e.local())),
        };
        self.status = r.unwrap_or_else(|e| format!("ERROR: {e}"));
        self.input.clear();
        self.modes.clear();
    }

    pub fn draw(&mut self, ui: &mut egui::Ui) {
        let t = self.theme;
        egui::Panel::top("diary_header")
            .exact_size(t.bar_thin + t.main_gap)
            .frame(egui::Frame::new().fill(t.bg).inner_margin(t.frame_gap))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    lcars_button(
                        ui,
                        &t,
                        "Diary",
                        palette::TAN,
                        egui::vec2(110.0, t.bar_thin),
                        Cap::Left,
                        true,
                    );
                    let seg = egui::vec2(90.0, t.bar_thin);
                    if lcars_button(ui, &t, "Today", palette::TEAL, seg, Cap::None, true).clicked()
                    {
                        self.date = chrono::Local::now().format("%Y-%m-%d").to_string();
                    }
                    if lcars_button(ui, &t, "All", palette::SLATE, seg, Cap::None, true).clicked() {
                        self.date.clear();
                        self.search.clear();
                    }
                    ui.add_space(t.main_gap);
                    ui.label(
                        egui::RichText::new("PERSONAL LOG")
                            .size(t.title_size)
                            .color(t.text),
                    );
                });
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(t.bg).inner_margin(t.frame_gap))
            .show(ui, |ui| {
                let full = ui.available_rect_before_wrap();
                let split = full.left() + full.width() * 0.45;
                let left = egui::Rect::from_min_max(full.min, egui::pos2(split, full.bottom()));
                let right =
                    egui::Rect::from_min_max(egui::pos2(split + t.main_gap, full.top()), full.max);
                ui.scope_builder(egui::UiBuilder::new().max_rect(left), |ui| {
                    let mut f = LcarsFrame::new(&t, Side::Left, 0.6);
                    f.min_height = left.height();
                    f.show(ui, &t, |ui| self.left_panel(ui));
                });
                ui.scope_builder(egui::UiBuilder::new().max_rect(right), |ui| {
                    let mut f = LcarsFrame::new(&t, Side::Right, 0.6);
                    f.top_color = palette::PLUM;
                    f.bottom_color = palette::OLIVE;
                    f.min_height = right.height();
                    f.show(ui, &t, |ui| self.entries_panel(ui));
                });
            });
    }

    fn left_panel(&mut self, ui: &mut egui::Ui) {
        let t = self.theme;
        let title = if self.editing.is_some() {
            "EDIT ENTRY"
        } else {
            "NEW ENTRY"
        };
        ui.label(egui::RichText::new(title).size(t.header_size));
        let input_id = ui.make_persistent_id("diary_input");
        let resp = ui.add(
            egui::TextEdit::singleline(&mut self.input)
                .id(input_id)
                .hint_text("Type and press Enter")
                .font(egui::TextStyle::Monospace)
                .desired_width(f32::INFINITY),
        );
        if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            self.save();
        }
        // Always-focused entry field, unless another widget (e.g. search) holds focus.
        if ui.ctx().memory(|m| m.focused().is_none()) {
            resp.request_focus();
        }
        readout(ui, &t, &self.status, self.status.starts_with("SAVED"));
        let b = egui::vec2(160.0, 40.0);
        ui.horizontal(|ui| {
            if self.editing.is_some() && action_button(ui, &t, "Cancel edit", b).clicked() {
                self.editing = None;
                self.input.clear();
                self.modes.clear();
            }
            if let Some(id) = self.selected() {
                if action_button(ui, &t, "Edit", b).clicked() {
                    self.modes.insert(id, Mode::Alt1);
                }
                if action_button(ui, &t, "Delete", b).clicked() {
                    self.confirm_delete = Some(id);
                }
            }
        });
        if let Some(id) = self.confirm_delete {
            ui.label(
                egui::RichText::new("DELETE THIS ENTRY?")
                    .size(t.header_size)
                    .color(Mode::Alt1.colors().label),
            );
            ui.horizontal(|ui| {
                if action_button(ui, &t, "Confirm delete", b).clicked() {
                    match self.store.delete(id) {
                        Ok(e) => {
                            self.undo = e;
                            self.status = "ENTRY DELETED".into();
                        }
                        Err(e) => self.status = format!("ERROR: {e}"),
                    }
                    self.modes.clear();
                    self.confirm_delete = None;
                }
                if action_button(ui, &t, "Cancel", b).clicked() {
                    self.confirm_delete = None;
                }
            });
        }
        if self.undo.is_some()
            && action_button(ui, &t, "Undo delete", b).clicked()
            && let Some(e) = self.undo.take()
        {
            self.status = match self.store.restore(&e) {
                Ok(()) => "DELETE UNDONE".into(),
                Err(e) => format!("ERROR: {e}"),
            };
        }
        ui.add_space(t.main_gap);
        ui.label(egui::RichText::new("FILTER").size(t.header_size));
        ui.add(
            egui::TextEdit::singleline(&mut self.search)
                .hint_text("Text contains")
                .desired_width(f32::INFINITY),
        );
        ui.add(
            egui::TextEdit::singleline(&mut self.date)
                .hint_text("Date YYYY-MM-DD (prefix)")
                .desired_width(f32::INFINITY),
        );
    }

    fn entries_panel(&mut self, ui: &mut egui::Ui) {
        let t = self.theme;
        let entries = self
            .store
            .list(&self.search, &self.date)
            .unwrap_or_default();
        ui.label(egui::RichText::new(format!("ENTRIES ({})", entries.len())).size(t.header_size));
        let mut rows: Vec<(String, Mode)> = entries
            .iter()
            .map(|e| {
                let mut text = e.text.replace('\n', " ");
                if text.chars().count() > 40 {
                    text = text.chars().take(39).collect::<String>() + "…";
                }
                (
                    format!("{}  {text}", e.local()),
                    self.modes.get(&e.id).copied().unwrap_or_default(),
                )
            })
            .collect();
        egui::ScrollArea::vertical().show(ui, |ui| mode_list(ui, &t, "entries", &mut rows, 28.0));
        for (e, (_, m)) in entries.iter().zip(rows) {
            let old = self.modes.get(&e.id).copied().unwrap_or_default();
            if m == old {
                continue;
            }
            self.modes.clear(); // one active row at a time
            if m != Mode::Inactive {
                self.modes.insert(e.id, m);
            }
            match m {
                Mode::Alt1 => {
                    self.editing = Some(e.id);
                    self.input = e.text.clone();
                }
                Mode::Alt2 => {
                    self.input = format!("Re {}: {}", e.local(), e.text);
                    self.editing = None;
                }
                _ => {
                    if self.editing == Some(e.id) {
                        self.editing = None;
                        self.input.clear();
                    }
                }
            }
        }
    }
}

impl eframe::App for Diary {
    fn ui(&mut self, ui: &mut egui::Ui, _f: &mut eframe::Frame) {
        self.draw(ui);
    }
}

fn db_path() -> std::path::PathBuf {
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        if a == "--db"
            && let Some(p) = args.next()
        {
            return p.into();
        }
    }
    directories::ProjectDirs::from("org", "rustcars", "rustcars-diary")
        .map(|d| d.data_dir().join("diary.sqlite"))
        .unwrap_or_else(|| "diary.sqlite".into())
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let path = db_path();
    let store =
        Store::open(&path).unwrap_or_else(|e| panic!("cannot open {}: {e}", path.display()));
    eprintln!("diary database: {}", path.display());
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 760.0])
            .with_title("rustcars diary"),
        ..Default::default()
    };
    eframe::run_native(
        "rustcars-diary",
        options,
        Box::new(move |cc| Ok(Box::new(Diary::new(&cc.egui_ctx, store)))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(test)]
mod preview {
    use super::*;
    #[test]
    fn render_diary_png() {
        let mut app: Option<Diary> = None;
        let mut harness = egui_kittest::Harness::builder()
            .with_size(egui::vec2(1180.0, 760.0))
            .wgpu()
            .build_ui(move |ui| {
                let d = app.get_or_insert_with(|| {
                    let s = Store::open_in_memory().unwrap();
                    let samples = [
                        (
                            "2026-10-20T13:05:00Z",
                            "Scanned 851.388 to 853.388, quiet on all channels",
                        ),
                        (
                            "2026-10-21T00:40:00Z",
                            "Night shift: Dalton repeater back on air",
                        ),
                        ("2026-10-21T14:15:00Z", "Recalibrated antenna, SWR 1.3"),
                        (
                            "2026-10-22T11:30:00Z",
                            "Coffee, then manual chapter 4 review",
                        ),
                        (
                            "2026-10-22T12:10:00Z",
                            "Pushed calculator and diary examples",
                        ),
                    ];
                    let ids: Vec<i64> = samples
                        .iter()
                        .map(|(t, x)| s.add(x, t).unwrap().id)
                        .collect();
                    let mut d = Diary::new(ui.ctx(), s);
                    d.modes.insert(ids[3], Mode::Primary);
                    d.confirm_delete = Some(ids[3]);
                    d.input = "Logged a new contact at 852.900".into();
                    d.status = "SAVED 2026-10-22 08:10".into();
                    d
                });
                d.draw(ui);
            });
        harness.run_steps(3);
        let mut img = harness.render().expect("render");
        for px in img.pixels_mut() {
            let a = px[3] as u32;
            for k in 0..3 {
                px[k] = ((px[k] as u32 * a + 0x1B * (255 - a)) / 255) as u8;
            }
            px[3] = 255;
        }
        img.save(concat!(env!("CARGO_MANIFEST_DIR"), "/docs/diary.png"))
            .unwrap();
    }
}
