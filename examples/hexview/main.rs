//! Hex viewer built from rustcars widgets.
//! Run: `cargo run --example hexview [-- FILE]` (desktop only; no file = a built-in sample, the
//! Antonio font). Files over 16 MiB are cut at 16 MiB and the title says so.
//!
//! Rows are a virtualised `mode_list`. The click modes are three ways of *marking* a row
//! (AGENTS.md P2):
//! * click   = inspect: the INSPECTOR on the right reads the bytes at that row.
//! * double  = bookmark: the row stays orange and is listed under BOOKMARKS for a quick jump.
//! * triple  = copy: the row's hex bytes go to the clipboard and the row stays purple.
//!
//! Also: GO TO an offset (`496` or `0x1F0`), FIND hex bytes (`DE AD`) or `"text"`.
//! The logic is in `hex.rs` and is unit tested.
mod hex;

use hex::ROW;
use rustcars::{
    Cap, LcarsFrame, LcarsTheme, Mode, Side, action_button, apply_theme, install_lcars_font,
    lcars_button, mode_list, palette, readout,
};
use std::collections::BTreeMap;

const LIMIT: usize = 16 * 1024 * 1024;
const ROW_H: f32 = 24.0;
const GAP: f32 = 2.0;

pub struct HexView {
    pub theme: LcarsTheme,
    pub name: String,
    pub data: Vec<u8>,
    pub truncated: bool,
    /// Row index -> mode, for rows that are not inactive.
    pub modes: BTreeMap<usize, Mode>,
    pub selected: Option<usize>,
    pub goto: String,
    pub find: String,
    pub scroll_to: Option<usize>,
    pub status: String,
}

impl HexView {
    pub fn new(ctx: &egui::Context, name: String, data: Vec<u8>, truncated: bool) -> Self {
        let theme = LcarsTheme::default();
        install_lcars_font(ctx);
        apply_theme(ctx, &theme);
        Self {
            theme,
            name,
            data,
            truncated,
            modes: BTreeMap::new(),
            selected: None,
            goto: String::new(),
            find: String::new(),
            scroll_to: None,
            status: "READY".into(),
        }
    }

    fn row_bytes(&self, row: usize) -> &[u8] {
        let s = (row * ROW).min(self.data.len());
        &self.data[s..(s + ROW).min(self.data.len())]
    }

    fn bookmarks(&self) -> Vec<usize> {
        self.modes
            .iter()
            .filter(|(_, m)| **m == Mode::Alt1)
            .map(|(r, _)| *r)
            .collect()
    }

    /// Select the byte at `off`: marks its row (click mode) and scrolls to it.
    pub fn select(&mut self, off: usize) {
        let row = off / ROW;
        self.modes.retain(|_, m| *m != Mode::Primary);
        self.modes.insert(row, Mode::Primary);
        self.selected = Some(off);
        self.scroll_to = Some(row);
    }

    fn do_goto(&mut self) {
        self.status = match hex::parse_offset(&self.goto) {
            Ok(o) if o < self.data.len() => {
                self.select(o);
                format!("AT 0x{o:06X}")
            }
            Ok(o) => format!("ERROR: 0x{o:X} is past the end (0x{:X})", self.data.len()),
            Err(e) => format!("ERROR: {e}"),
        };
    }

    fn do_find(&mut self) {
        self.status = match hex::parse_pattern(&self.find) {
            Ok(p) => {
                let from = self.selected.map_or(0, |s| s + 1);
                match hex::find(&self.data, &p, from) {
                    Some(o) => {
                        self.select(o);
                        format!("FOUND AT 0x{o:06X}")
                    }
                    None => "NOT FOUND".into(),
                }
            }
            Err(e) => format!("ERROR: {e}"),
        };
    }

    pub fn draw(&mut self, ui: &mut egui::Ui) {
        let t = self.theme;
        egui::Panel::top("hex_header")
            .exact_size(t.bar_thin + t.main_gap)
            .frame(egui::Frame::new().fill(t.bg).inner_margin(t.frame_gap))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    lcars_button(
                        ui,
                        &t,
                        "Hex",
                        palette::TAN,
                        egui::vec2(110.0, t.bar_thin),
                        Cap::Left,
                        true,
                    );
                    ui.add_space(t.main_gap);
                    ui.label(
                        egui::RichText::new("HEX VIEWER")
                            .size(t.title_size)
                            .color(t.text),
                    );
                });
            });
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(t.bg).inner_margin(t.frame_gap))
            .show(ui, |ui| {
                let full = ui.available_rect_before_wrap();
                let split = full.left() + 830.0;
                let left = egui::Rect::from_min_max(full.min, egui::pos2(split, full.bottom()));
                let right =
                    egui::Rect::from_min_max(egui::pos2(split + t.main_gap, full.top()), full.max);
                ui.scope_builder(egui::UiBuilder::new().max_rect(left), |ui| {
                    let mut f = LcarsFrame::new(&t, Side::Left, 0.6);
                    f.min_height = left.height();
                    f.show(ui, &t, |ui| self.rows_panel(ui));
                });
                ui.scope_builder(egui::UiBuilder::new().max_rect(right), |ui| {
                    let mut f = LcarsFrame::new(&t, Side::Right, 0.6);
                    f.top_color = palette::PLUM;
                    f.bottom_color = palette::SLATE;
                    f.min_height = right.height();
                    f.show(ui, &t, |ui| self.side_panel(ui));
                });
            });
    }

    fn rows_panel(&mut self, ui: &mut egui::Ui) {
        let t = self.theme;
        let rows = hex::row_count(self.data.len());
        let cut = if self.truncated {
            " (CUT AT 16 MIB)"
        } else {
            ""
        };
        ui.label(
            egui::RichText::new(format!(
                "{}: {} BYTES{cut}",
                self.name.to_uppercase(),
                self.data.len()
            ))
            .size(t.header_size),
        );
        ui.label(
            egui::RichText::new("CLICK INSPECTS · DOUBLE BOOKMARKS · TRIPLE COPIES THE ROW")
                .size(t.data_size)
                .color(palette::TEXT_MUTED),
        );
        // show_rows measures a row as height + item spacing: keep it equal to mode_list's gap
        ui.spacing_mut().item_spacing.y = GAP;
        let mut area = egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(ui.available_height() - 4.0);
        if let Some(r) = self.scroll_to.take() {
            area = area.vertical_scroll_offset((r.saturating_sub(2)) as f32 * (ROW_H + GAP));
        }
        let out = area.show_rows(ui, ROW_H, rows, |ui, range| {
            let mut list: Vec<(String, Mode)> = range
                .clone()
                .map(|r| {
                    (
                        hex::format_row(r * ROW, self.row_bytes(r)),
                        self.modes.get(&r).copied().unwrap_or_default(),
                    )
                })
                .collect();
            mode_list(ui, &t, "hex_rows", &mut list, ROW_H);
            (range, list)
        });
        let (range, list) = out.inner;
        for (k, (_, m)) in list.iter().enumerate() {
            let row = range.start + k;
            let old = self.modes.get(&row).copied().unwrap_or_default();
            if *m == old {
                continue;
            }
            match m {
                Mode::Inactive => {
                    self.modes.remove(&row);
                    if self.selected.map(|s| s / ROW) == Some(row) {
                        self.selected = None;
                    }
                }
                _ => {
                    // inspect and copy keep one row each; bookmarks accumulate
                    if matches!(m, Mode::Primary | Mode::Alt2) {
                        self.modes.retain(|_, x| x != m);
                    }
                    self.modes.insert(row, *m);
                }
            }
            match m {
                Mode::Primary => self.selected = Some(row * ROW),
                Mode::Alt1 => self.status = format!("BOOKMARKED 0x{:06X}", row * ROW),
                Mode::Alt2 => {
                    let text = hex::row_hex(self.row_bytes(row));
                    ui.ctx().copy_text(text.clone());
                    self.status = format!("COPIED: {text}");
                }
                Mode::Inactive => {}
            }
        }
    }

    fn side_panel(&mut self, ui: &mut egui::Ui) {
        let t = self.theme;
        ui.label(egui::RichText::new("GO TO").size(t.header_size));
        let r = ui.add(
            egui::TextEdit::singleline(&mut self.goto)
                .hint_text("496 or 0x1F0")
                .font(egui::TextStyle::Monospace)
                .desired_width(f32::INFINITY),
        );
        if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            self.do_goto();
        }
        ui.label(egui::RichText::new("FIND").size(t.header_size));
        let r = ui.add(
            egui::TextEdit::singleline(&mut self.find)
                .hint_text("DE AD BE EF or \"RIFF\"")
                .font(egui::TextStyle::Monospace)
                .desired_width(f32::INFINITY),
        );
        let enter = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        ui.horizontal(|ui| {
            let b = egui::vec2(110.0, 36.0);
            if action_button(ui, &t, "Go to", b).clicked() {
                self.do_goto();
            }
            if action_button(ui, &t, "Find next", b).clicked() || enter {
                self.do_find();
            }
        });
        readout(ui, &t, &self.status, self.status.starts_with("FOUND"));
        ui.add_space(t.main_gap / 2.0);
        ui.label(egui::RichText::new("INSPECTOR").size(t.header_size));
        match self.selected {
            Some(off) => {
                readout(ui, &t, &format!("OFFSET 0x{off:06X} ({off})"), true);
                egui::Grid::new("inspect")
                    .spacing(egui::vec2(t.main_gap, t.frame_gap))
                    .show(ui, |ui| {
                        for (k, v) in hex::inspect(&self.data, off) {
                            ui.label(egui::RichText::new(k).color(palette::TEXT_MUTED));
                            readout(ui, &t, &v, false);
                            ui.end_row();
                        }
                    });
            }
            None => {
                ui.label(
                    egui::RichText::new("CLICK A ROW, OR USE GO TO").color(palette::TEXT_MUTED),
                );
            }
        }
        ui.add_space(t.main_gap / 2.0);
        ui.label(egui::RichText::new("BOOKMARKS").size(t.header_size));
        let marks = self.bookmarks();
        if marks.is_empty() {
            ui.label(egui::RichText::new("DOUBLE-CLICK A ROW").color(palette::TEXT_MUTED));
        }
        ui.horizontal_wrapped(|ui| {
            for row in marks.into_iter().take(12) {
                let label = format!("{:06X}", row * ROW);
                if action_button(ui, &t, &label, egui::vec2(96.0, 32.0)).clicked() {
                    self.scroll_to = Some(row);
                }
            }
        });
    }
}

impl eframe::App for HexView {
    fn ui(&mut self, ui: &mut egui::Ui, _f: &mut eframe::Frame) {
        self.draw(ui);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let (name, data, truncated) = match std::env::args().nth(1) {
        Some(p) => {
            let path = std::path::Path::new(&p);
            match hex::load(path, LIMIT) {
                Ok((d, t)) => (
                    path.file_name()
                        .map_or(p.clone(), |n| n.to_string_lossy().into()),
                    d,
                    t,
                ),
                Err(e) => {
                    eprintln!("cannot read {p}: {e}");
                    std::process::exit(1);
                }
            }
        }
        None => (
            "Antonio-Variable.ttf (sample)".into(),
            rustcars::ANTONIO_TTF.to_vec(),
            false,
        ),
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 760.0])
            .with_title("rustcars hex viewer"),
        ..Default::default()
    };
    eframe::run_native(
        "rustcars-hexview",
        options,
        Box::new(move |cc| Ok(Box::new(HexView::new(&cc.egui_ctx, name, data, truncated)))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(test)]
mod preview {
    use super::*;

    fn sample(ctx: &egui::Context) -> HexView {
        HexView::new(
            ctx,
            "Antonio-Variable.ttf (sample)".into(),
            rustcars::ANTONIO_TTF.to_vec(),
            false,
        )
    }

    #[test]
    fn render_hexview_png() {
        let mut app: Option<HexView> = None;
        let mut harness = egui_kittest::Harness::builder()
            .with_size(egui::vec2(1280.0, 760.0))
            .wgpu()
            .build_ui(move |ui| {
                let a = app.get_or_insert_with(|| {
                    let mut a = sample(ui.ctx());
                    a.select(0x20);
                    a.modes.insert(3, Mode::Alt1);
                    a.modes.insert(8, Mode::Alt1);
                    a.modes.insert(6, Mode::Alt2);
                    a.find = "\"GDEF\"".into();
                    a.status = "FOUND AT 0x00000B".into();
                    a.scroll_to = None;
                    a
                });
                a.draw(ui);
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
        img.save(concat!(env!("CARGO_MANIFEST_DIR"), "/docs/hexview.png"))
            .unwrap();
    }

    #[test]
    fn goto_find_and_marks() {
        let ctx = egui::Context::default();
        let mut a = HexView::new(
            &ctx,
            "t".into(),
            (0..=255u8).cycle().take(1000).collect(),
            false,
        );
        a.goto = "0x1F0".into();
        a.do_goto();
        assert_eq!(a.selected, Some(496));
        assert_eq!(a.modes.get(&31), Some(&Mode::Primary));
        assert_eq!(a.scroll_to, Some(31));
        a.goto = "5000".into();
        a.do_goto();
        assert!(a.status.starts_with("ERROR"));
        assert_eq!(a.selected, Some(496), "a failed go to keeps the selection");
        // one Primary row at a time
        a.select(16);
        assert_eq!(a.modes.values().filter(|m| **m == Mode::Primary).count(), 1);
        // find starts after the selection, then wraps
        a.find = "0A 0B".into();
        a.do_find();
        assert_eq!(a.selected, Some(266), "first match after the selection");
        a.do_find();
        assert_eq!(a.selected, Some(522), "next match, 256 bytes later");
        a.do_find();
        assert_eq!(a.selected, Some(778));
        a.do_find();
        assert_eq!(a.selected, Some(10), "wraps to the start");
        a.find = "\"zzz\"".into();
        a.do_find();
        assert_eq!(a.status, "NOT FOUND");
        a.modes.insert(2, Mode::Alt1);
        a.modes.insert(9, Mode::Alt1);
        assert_eq!(a.bookmarks(), vec![2, 9]);
        assert_eq!(
            a.row_bytes(62).len(),
            8,
            "short last row (1000 = 62 rows + 8)"
        );
    }
}
