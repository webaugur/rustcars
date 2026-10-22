//! rustcars: LCARS-style widgets for egui 0.37.
//! The "paint-behind" frame technique (reserve a `Shape::Noop` slot, lay out content,
//! then `painter.set` the chrome sized to the real content rect) is inspired by
//! paxel/dedup-rs `crates/dedup-gui/src/lcars.rs` (MIT, https://github.com/paxel/dedup-rs).
use egui::{
    Align2, Color32, Context, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Rect,
    Response, Sense, Shadow, Shape, Stroke, TextStyle, Ui, UiBuilder, Vec2, pos2, vec2,
};
use std::sync::Arc;

// ---------------------------------------------------------------- theme
#[derive(Clone, Copy)]
pub struct LcarsTheme {
    pub bg: Color32,
    pub primary: Color32,
    pub secondary: Color32,
    pub accent: Color32,
    pub alert: Color32,
    pub text: Color32,
    pub text_hi: Color32,
    pub bar_thick: f32,
    pub bar_thin: f32,
    pub frame_gap: f32,
    pub main_gap: f32,
    pub title_size: f32,
    pub header_size: f32,
    pub data_size: f32,
}

impl Default for LcarsTheme {
    fn default() -> Self {
        Self {
            bg: Color32::BLACK,
            primary: Color32::from_rgb(0xFF, 0x99, 0x00),
            secondary: Color32::from_rgb(0xCC, 0x99, 0xCC),
            accent: Color32::from_rgb(0x99, 0x99, 0xFF),
            alert: Color32::from_rgb(0xCC, 0x44, 0x44),
            text: Color32::from_rgb(0xFF, 0x99, 0x00),
            text_hi: Color32::from_rgb(0xFF, 0xCC, 0x99),
            bar_thick: 120.0,
            bar_thin: 40.0,
            frame_gap: 4.0,
            main_gap: 16.0,
            title_size: 36.0,
            header_size: 22.0,
            data_size: 15.0,
        }
    }
}

/// Tint (t>0, toward white) or shade (t<0, toward black) of the same hue.
pub fn tint(c: Color32, t: f32) -> Color32 {
    let f = |v: u8| -> u8 {
        let v = v as f32;
        (if t >= 0.0 {
            v + (255.0 - v) * t
        } else {
            v * (1.0 + t)
        })
        .round() as u8
    };
    Color32::from_rgb(f(c.r()), f(c.g()), f(c.b()))
}

/// WCAG relative luminance of an sRGB color.
pub fn luminance(c: Color32) -> f32 {
    let ch = |v: u8| {
        let s = v as f32 / 255.0;
        if s <= 0.03928 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * ch(c.r()) + 0.7152 * ch(c.g()) + 0.0722 * ch(c.b())
}

/// Auto ink: black on light fills, white on dark fills (whichever has higher contrast).
pub fn ink_on(fill: Color32) -> Color32 {
    let l = luminance(fill);
    let vs_black = (l + 0.05) / 0.05;
    let vs_white = 1.05 / (l + 0.05);
    if vs_black >= vs_white {
        Color32::BLACK
    } else {
        Color32::WHITE
    }
}

// ---------------------------------------------------------------- fonts & style
/// Bundled Antonio (SIL OFL 1.1). Keep assets/fonts/OFL.txt + FONTS.md alongside it.
pub const ANTONIO_TTF: &[u8] = include_bytes!("../assets/fonts/Antonio-Variable.ttf");

pub fn install_font(ctx: &Context, ttf: &'static [u8]) {
    let mut fonts = FontDefinitions::default();
    fonts
        .font_data
        .insert("lcars".to_owned(), Arc::new(FontData::from_static(ttf)));
    fonts
        .families
        .get_mut(&FontFamily::Proportional)
        .unwrap()
        .insert(0, "lcars".to_owned());
    ctx.set_fonts(fonts); // egui defaults stay as fallbacks
}

pub fn install_lcars_font(ctx: &Context) {
    install_font(ctx, ANTONIO_TTF);
}

pub fn apply_theme(ctx: &Context, t: &LcarsTheme) {
    ctx.global_style_mut(|s| {
        use FontFamily::{Monospace, Proportional};
        s.text_styles = [
            (TextStyle::Heading, FontId::new(t.title_size, Proportional)),
            (TextStyle::Button, FontId::new(t.header_size, Proportional)),
            (TextStyle::Body, FontId::new(t.data_size, Proportional)),
            (TextStyle::Small, FontId::new(t.data_size, Proportional)),
            (TextStyle::Monospace, FontId::new(t.data_size, Monospace)),
        ]
        .into();
        s.spacing.item_spacing = vec2(t.frame_gap, t.frame_gap);
        s.spacing.button_padding = vec2(t.main_gap, t.frame_gap);
        let v = &mut s.visuals;
        v.dark_mode = true;
        v.panel_fill = t.bg;
        v.window_fill = t.bg;
        v.extreme_bg_color = t.bg;
        v.window_shadow = Shadow::NONE;
        v.popup_shadow = Shadow::NONE;
        v.window_stroke = Stroke::NONE;
        v.override_text_color = Some(t.text);
        v.selection.bg_fill = t.accent;
        for w in [
            &mut v.widgets.noninteractive,
            &mut v.widgets.inactive,
            &mut v.widgets.hovered,
            &mut v.widgets.active,
            &mut v.widgets.open,
        ] {
            w.bg_stroke = Stroke::NONE;
            w.corner_radius = CornerRadius::same(255);
        }
    });
}

// ---------------------------------------------------------------- caps & buttons
#[derive(Clone, Copy, PartialEq)]
pub enum Cap {
    Both,
    Left,
    Right,
    None,
}

pub fn cap_radius(cap: Cap, h: f32) -> CornerRadius {
    let r = (h / 2.0).min(255.0) as u8;
    match cap {
        Cap::Both => CornerRadius::same(r),
        Cap::Left => CornerRadius {
            nw: r,
            sw: r,
            ne: 0,
            se: 0,
        },
        Cap::Right => CornerRadius {
            nw: 0,
            sw: 0,
            ne: r,
            se: r,
        },
        Cap::None => CornerRadius::ZERO,
    }
}

/// Pill / segment button. Uppercase label, right-bottom aligned, auto ink color.
pub fn lcars_button(
    ui: &mut Ui,
    t: &LcarsTheme,
    label: &str,
    base: Color32,
    size: Vec2,
    cap: Cap,
    enabled: bool,
) -> Response {
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, resp) = ui.allocate_exact_size(size, sense);
    let fill = if !enabled {
        tint(base, -0.6)
    } else if resp.is_pointer_button_down_on() {
        tint(base, -0.25)
    } else if resp.hovered() {
        tint(base, 0.3)
    } else {
        base
    };
    let p = ui.painter();
    p.rect_filled(rect, cap_radius(cap, rect.height()), fill);
    if resp.has_focus() {
        p.rect_stroke(
            rect.shrink(2.0),
            cap_radius(cap, rect.height()),
            Stroke::new(2.0, t.text_hi),
            egui::StrokeKind::Inside,
        );
    }
    let pad = (rect.height() / 2.0).min(t.main_gap);
    p.text(
        pos2(rect.right() - pad, rect.bottom() - t.frame_gap),
        Align2::RIGHT_BOTTOM,
        label.to_uppercase(),
        FontId::proportional(t.header_size),
        ink_on(fill),
    );
    resp
}

// ---------------------------------------------------------------- elbows
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Corner {
    TL,
    TR,
    BL,
    BR,
}

/// Elbow specification. Geometry is defined once for TL and mirrored for TR/BL/BR.
/// Vertical bar width `thick` runs along the left/right edge; horizontal bar height `thin`
/// along the top/bottom edge. `r_out` = outer sweep radius, `r_in` = concave inner radius.
/// `bg` must be the interior background (it carves the concave curve).
#[derive(Clone, Copy, Debug)]
pub struct Elbow {
    pub corner: Corner,
    pub thick: f32,
    pub thin: f32,
    pub r_out: f32,
    pub r_in: f32,
    pub color: Color32,
    pub bg: Color32,
}

impl Elbow {
    /// Suggested defaults: r_out = 0.75 * thick, r_in = thin.
    pub fn new(corner: Corner, thick: f32, thin: f32, color: Color32, bg: Color32) -> Self {
        Self {
            corner,
            thick,
            thin,
            r_out: thick * 0.75,
            r_in: thin,
            color,
            bg,
        }
    }

    /// Paint immediately.
    pub fn paint(&self, ui: &Ui, rect: Rect) {
        ui.painter().extend(elbow_shapes(rect, self));
    }
}

/// Elbow geometry as shapes (for paint-behind use with `Painter::set`).
pub fn elbow_shapes(rect: Rect, e: &Elbow) -> Vec<Shape> {
    let Elbow {
        corner,
        thick,
        thin,
        r_out,
        r_in,
        color,
        bg,
    } = *e;
    debug_assert!(
        (thick - thin).abs() > 0.5,
        "LCARS: thickness must change at a turn"
    );
    let (fx, fy) = match corner {
        Corner::TL => (false, false),
        Corner::TR => (true, false),
        Corner::BL => (false, true),
        Corner::BR => (true, true),
    };
    let map = |r: Rect| -> Rect {
        let (x0, x1) = if fx {
            (rect.right() - r.max.x, rect.right() - r.min.x)
        } else {
            (rect.left() + r.min.x, rect.left() + r.max.x)
        };
        let (y0, y1) = if fy {
            (rect.bottom() - r.max.y, rect.bottom() - r.min.y)
        } else {
            (rect.top() + r.min.y, rect.top() + r.max.y)
        };
        Rect::from_min_max(pos2(x0, y0), pos2(x1, y1))
    };
    let one = |r: f32| -> CornerRadius {
        let r = r.clamp(0.0, 255.0) as u8;
        let mut c = CornerRadius::ZERO;
        match corner {
            Corner::TL => c.nw = r,
            Corner::TR => c.ne = r,
            Corner::BL => c.sw = r,
            Corner::BR => c.se = r,
        }
        c
    };
    let (w, h) = (rect.width(), rect.height());
    // epaint clamps radii to half the rect's shorter side: size the swept block so r_out fits.
    let bw = (thick + r_in).max(2.0 * r_out).min(w);
    let bh = (thin + r_in).max(2.0 * r_out).min(h);
    let mut v = vec![Shape::rect_filled(
        map(Rect::from_min_max(pos2(0.0, 0.0), pos2(bw, bh))),
        one(r_out),
        color,
    )];
    if w > bw {
        v.push(Shape::rect_filled(
            map(Rect::from_min_max(pos2(bw, 0.0), pos2(w, thin))),
            0.0,
            color,
        ));
    }
    if h > bh {
        v.push(Shape::rect_filled(
            map(Rect::from_min_max(pos2(0.0, bh), pos2(thick, h))),
            0.0,
            color,
        ));
    }
    v.push(Shape::rect_filled(
        map(Rect::from_min_max(
            pos2(thick, thin),
            pos2(bw + 1.0, bh + 1.0),
        )),
        one(r_in),
        bg,
    ));
    v
}

/// Convenience wrapper around [`Elbow::paint`].
pub fn paint_elbow(ui: &Ui, rect: Rect, e: &Elbow) {
    e.paint(ui, rect);
}

#[derive(Clone, Copy, PartialEq)]
pub enum Side {
    Left,
    Right,
}

/// A C-shaped (or reversed-C) LCARS frame: top + bottom elbows on `side`, each horizontal
/// bar terminated by a rounded cap segment. Chrome is painted *behind* the content and
/// sized to the content's real rect (paint-behind technique, see module docs).
pub struct LcarsFrame {
    pub side: Side,
    pub top_color: Color32,
    pub bottom_color: Color32,
    pub thick: f32,
    pub thin: f32,
    pub cap_len: f32,
    pub min_height: f32,
}

impl LcarsFrame {
    pub fn new(t: &LcarsTheme, side: Side, scale: f32) -> Self {
        Self {
            side,
            top_color: t.primary,
            bottom_color: t.secondary,
            thick: t.bar_thick * scale,
            thin: t.bar_thin * scale,
            cap_len: 48.0,
            min_height: 0.0,
        }
    }

    pub fn show<R>(
        &self,
        ui: &mut Ui,
        t: &LcarsTheme,
        add: impl FnOnce(&mut Ui) -> R,
    ) -> (R, Rect) {
        let (thick, thin) = (self.thick, self.thin);
        let (r_out, r_in) = (thick * 0.75, thin);
        // 1) reserve a paint slot behind everything the content will draw
        let bg_idx = ui.painter().add(Shape::Noop);
        // 2) lay out content inside the frame's interior. (Manual margins: egui's Margin is
        //    i8, too small for 120+ px bars, so we use scope_builder with a shrunk max_rect.)
        let outer = ui.available_rect_before_wrap();
        let pad_side = thick + t.main_gap;
        let pad_v = thin + t.main_gap;
        let inner_max = match self.side {
            Side::Left => Rect::from_min_max(
                outer.min + vec2(pad_side, pad_v),
                pos2(outer.right() - t.main_gap, outer.bottom() - pad_v),
            ),
            Side::Right => Rect::from_min_max(
                outer.min + vec2(t.main_gap, pad_v),
                pos2(outer.right() - pad_side, outer.bottom() - pad_v),
            ),
        };
        let inner = ui.scope_builder(UiBuilder::new().max_rect(inner_max), |ui| {
            ui.set_min_width(ui.available_width()); // claim full width so bars span the panel
            add(ui)
        });
        let used = inner.response.rect;
        let height = (used.height() + 2.0 * pad_v)
            .max(self.min_height)
            .max(4.0 * r_out + t.frame_gap);
        let frame_rect = Rect::from_min_size(outer.min, vec2(outer.width(), height));
        ui.allocate_rect(frame_rect, Sense::hover());
        // 3) build the chrome now that the real size is known, and set it into the reserved slot
        let left = self.side == Side::Left;
        let mid = frame_rect.center().y;
        let halves = [
            (
                Rect::from_min_max(
                    frame_rect.min,
                    pos2(frame_rect.right(), mid - t.frame_gap / 2.0),
                ),
                true,
                self.top_color,
            ),
            (
                Rect::from_min_max(
                    pos2(frame_rect.left(), mid + t.frame_gap / 2.0),
                    frame_rect.max,
                ),
                false,
                self.bottom_color,
            ),
        ];
        let mut shapes = vec![Shape::rect_filled(frame_rect, 0.0, t.bg)];
        for (r, top, col) in halves {
            let corner = match (left, top) {
                (true, true) => Corner::TL,
                (true, false) => Corner::BL,
                (false, true) => Corner::TR,
                (false, false) => Corner::BR,
            };
            let er = if left {
                Rect::from_min_max(
                    r.min,
                    pos2(r.right() - self.cap_len - t.frame_gap, r.bottom()),
                )
            } else {
                Rect::from_min_max(pos2(r.left() + self.cap_len + t.frame_gap, r.top()), r.max)
            };
            shapes.extend(elbow_shapes(
                er,
                &Elbow {
                    corner,
                    thick,
                    thin,
                    r_out,
                    r_in,
                    color: col,
                    bg: t.bg,
                },
            ));
            let y0 = if top { r.top() } else { r.bottom() - thin };
            let x0 = if left {
                r.right() - self.cap_len
            } else {
                r.left()
            };
            let cap = Rect::from_min_size(pos2(x0, y0), vec2(self.cap_len, thin));
            shapes.push(Shape::rect_filled(
                cap,
                cap_radius(if left { Cap::Right } else { Cap::Left }, thin),
                col,
            ));
        }
        ui.painter().set(bg_idx, Shape::Vec(shapes));
        (inner.inner, frame_rect)
    }
}

// ---------------------------------------------------------------- readouts
pub fn readout(ui: &mut Ui, t: &LcarsTheme, value: &str, highlight: bool) {
    let c = if highlight { t.text_hi } else { t.text };
    ui.label(egui::RichText::new(value).color(c).size(t.data_size));
}

// ---------------------------------------------------------------- demo
pub fn demo(ui: &mut Ui, t: &LcarsTheme) {
    egui::Panel::top("lcars_header")
        .exact_size(t.bar_thin + t.main_gap)
        .frame(egui::Frame::new().fill(t.bg).inner_margin(t.frame_gap))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                lcars_button(
                    ui,
                    t,
                    "Main",
                    t.primary,
                    vec2(140.0, t.bar_thin),
                    Cap::Left,
                    true,
                );
                for (l, c) in [
                    ("Nav", t.secondary),
                    ("Comm", t.accent),
                    ("Ops", t.secondary),
                ] {
                    lcars_button(ui, t, l, c, vec2(110.0, t.bar_thin), Cap::None, true);
                }
                ui.add_space(t.main_gap);
                ui.label(
                    egui::RichText::new("LCARS 47")
                        .size(t.title_size)
                        .color(t.primary),
                );
                lcars_button(
                    ui,
                    t,
                    "",
                    t.primary,
                    vec2(60.0, t.bar_thin),
                    Cap::Right,
                    true,
                );
            });
        });
    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(t.bg).inner_margin(t.frame_gap))
        .show(ui, |ui| {
            let time = ui.input(|i| i.time);
            let full = ui.available_rect_before_wrap();
            let split = full.left() + full.width() * 0.6;
            let left_r =
                Rect::from_min_max(full.min, pos2(split - t.main_gap / 2.0, full.bottom()));
            let right_r = Rect::from_min_max(pos2(split + t.main_gap / 2.0, full.top()), full.max);

            ui.scope_builder(UiBuilder::new().max_rect(left_r), |ui| {
                let mut f = LcarsFrame::new(t, Side::Left, 1.0);
                f.min_height = left_r.height();
                let (_, fr) = f.show(ui, t, |ui| {
                    ui.label(egui::RichText::new("SENSOR TELEMETRY").size(t.header_size));
                    egui::Grid::new("nums")
                        .spacing(vec2(t.main_gap, t.frame_gap))
                        .show(ui, |ui| {
                            for row in 0..6u64 {
                                for col in 0..4u64 {
                                    let v = ((time * 1000.0) as u64 / (230 + 80 * col)
                                        + row * 7919
                                        + col * 104729)
                                        % 100000;
                                    readout(ui, t, &format!("{v:05}"), row == 2 && col == 1);
                                }
                                ui.end_row();
                            }
                        });
                });
                // frame segments as buttons, cut into the left vertical bar
                let side = Rect::from_min_size(
                    pos2(fr.left(), fr.center().y - 70.0),
                    vec2(t.bar_thick, 140.0),
                );
                ui.painter()
                    .rect_filled(side.expand2(vec2(0.0, t.frame_gap)), 0.0, t.bg);
                ui.scope_builder(UiBuilder::new().max_rect(side), |ui| {
                    ui.spacing_mut().item_spacing = vec2(0.0, t.frame_gap);
                    lcars_button(
                        ui,
                        t,
                        "Sensors",
                        t.accent,
                        vec2(t.bar_thick, 44.0),
                        Cap::None,
                        true,
                    );
                    lcars_button(
                        ui,
                        t,
                        "Shields",
                        t.secondary,
                        vec2(t.bar_thick, 44.0),
                        Cap::None,
                        true,
                    );
                    lcars_button(
                        ui,
                        t,
                        "Off",
                        t.secondary,
                        vec2(t.bar_thick, 44.0),
                        Cap::None,
                        false,
                    );
                });
            });

            ui.scope_builder(UiBuilder::new().max_rect(right_r), |ui| {
                let mut f = LcarsFrame::new(t, Side::Right, 0.6);
                f.min_height = right_r.height();
                f.top_color = t.accent;
                f.bottom_color = t.primary;
                f.show(ui, t, |ui| {
                    ui.label(egui::RichText::new("ENGINEERING").size(t.header_size));
                    lcars_button(ui, t, "Warp", t.primary, vec2(150.0, 44.0), Cap::Both, true);
                    lcars_button(ui, t, "Alert", t.alert, vec2(150.0, 44.0), Cap::Both, true);
                    readout(ui, t, "21.0473", false);
                });
            });
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(250));
        });
}
