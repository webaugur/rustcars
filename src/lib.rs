//! rustcars: LCARS-style widgets for egui 0.37.
//! The "paint-behind" frame technique (reserve a `Shape::Noop` slot, lay out content,
//! then `painter.set` the chrome sized to the real content rect) is inspired by
//! paxel/dedup-rs `crates/dedup-gui/src/lcars.rs` (MIT, https://github.com/paxel/dedup-rs).
use egui::{
    Align2, Color32, Context, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Rect,
    Response, Sense, Shadow, Shape, Stroke, TextStyle, Ui, UiBuilder, Vec2, pos2, vec2,
};
use std::sync::Arc;

#[cfg(all(feature = "web", target_arch = "wasm32"))]
pub mod web;

// ---------------------------------------------------------------- palette
/// [PROJECT] palette. Base hues sampled from the user's reference screenshots (IMG2 tabs,
/// IMG1 buttons, IMG3 lists). Frame hues are brightened variants of the sampled tab colors so
/// that bars/segments reach >= 3:1 against the background (WCAG 2.2 SC 1.4.11).
pub mod palette {
    use egui::Color32;
    /// Page background (sampled IMG1/IMG3 #1B1B1B).
    pub const BG: Color32 = Color32::from_rgb(0x1B, 0x1B, 0x1B);
    /// List row / surface (sampled IMG3 #202020).
    pub const SURFACE: Color32 = Color32::from_rgb(0x20, 0x20, 0x20);
    /// Tab strip (sampled IMG2 #303030).
    pub const STRIP: Color32 = Color32::from_rgb(0x30, 0x30, 0x30);
    // Frame / tab hues: (sampled -> UI variant with >= 3:1 vs BG)
    /// Tan "Tune" (sampled #9C805A, already 4.63:1).
    pub const TAN: Color32 = Color32::from_rgb(0x9C, 0x80, 0x5A);
    /// Slate blue "Plan" (sampled #3A4E66).
    pub const SLATE: Color32 = Color32::from_rgb(0x51, 0x6D, 0x8F);
    /// Olive "Channels" (sampled #56603A).
    pub const OLIVE: Color32 = Color32::from_rgb(0x64, 0x6F, 0x43);
    /// Brick "Meters" (sampled #70463E).
    pub const BRICK: Color32 = Color32::from_rgb(0x94, 0x5C, 0x52);
    /// Teal "Levels" (sampled #345A54).
    pub const TEAL: Color32 = Color32::from_rgb(0x43, 0x73, 0x6C);
    /// Plum "Config" (sampled #604666).
    pub const PLUM: Color32 = Color32::from_rgb(0x81, 0x5E, 0x89);
    /// Normal text (warm light).
    pub const TEXT: Color32 = Color32::from_rgb(0xD9, 0xC7, 0xA8);
    /// Muted text for inactive list rows (sampled IMG3 #8B8B8B, lightened).
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(0x9A, 0x9A, 0x9A);
}

/// Colors for one interaction state: dark tinted fill, 1px border, label.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StateColors {
    pub fill: Color32,
    pub border: Color32,
    pub label: Color32,
}

/// [PROJECT] click-count modes of one function.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash)]
pub enum Mode {
    /// Gray.
    #[default]
    Inactive,
    /// Single click: primary selection, green.
    Primary,
    /// Double click: alternate 1, orange.
    Alt1,
    /// Triple click: alternate 2, purple.
    Alt2,
}

impl Mode {
    pub const ALL: [Mode; 4] = [Mode::Inactive, Mode::Primary, Mode::Alt1, Mode::Alt2];

    /// Click count -> mode. 0 = Inactive; 4+ clamps to triple (Alt2); there is never a 4th mode.
    pub fn from_clicks(n: u32) -> Mode {
        match n {
            0 => Mode::Inactive,
            1 => Mode::Primary,
            2 => Mode::Alt1,
            _ => Mode::Alt2,
        }
    }

    /// Resolve a finished click gesture against the current mode.
    /// Rule: the gesture selects its mode; repeating the gesture of the *current* mode
    /// toggles the control off (Inactive). Different gesture = switch mode directly.
    pub fn after_gesture(self, clicks: u32) -> Mode {
        let m = Mode::from_clicks(clicks);
        if m == self { Mode::Inactive } else { m }
    }

    /// Number of indicator pips (non-color mode indicator): 0, 1, 2 or 3.
    /// Arrangements follow U+00B7 · (1), U+2236 ∶ (2, vertical pair), U+2056 ⁖ (3, triangle).
    pub fn pips(self) -> u8 {
        match self {
            Mode::Inactive => 0,
            Mode::Primary => 1,
            Mode::Alt1 => 2,
            Mode::Alt2 => 3,
        }
    }

    /// Short mode name for screen readers ("Scan, mode Alt 1").
    pub fn short_name(self) -> &'static str {
        match self {
            Mode::Inactive => "Off",
            Mode::Primary => "Select",
            Mode::Alt1 => "Alt 1",
            Mode::Alt2 => "Alt 2",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Mode::Inactive => "Off",
            Mode::Primary => "Select (click)",
            Mode::Alt1 => "Alt 1 (double-click / Shift)",
            Mode::Alt2 => "Alt 2 (triple-click / Ctrl)",
        }
    }

    /// State colors (IMG1-derived; orange added to fit). Ratios listed in AGENTS.md and tested.
    pub fn colors(self) -> StateColors {
        let c = |f: [u8; 3], b: [u8; 3], l: [u8; 3]| StateColors {
            fill: Color32::from_rgb(f[0], f[1], f[2]),
            border: Color32::from_rgb(b[0], b[1], b[2]),
            label: Color32::from_rgb(l[0], l[1], l[2]),
        };
        match self {
            Mode::Inactive => c([0x1C, 0x1C, 0x20], [0x7A, 0x7A, 0x82], [0xE8, 0xE4, 0xD8]),
            Mode::Primary => c([0x12, 0x3A, 0x1C], [0x2E, 0xB0, 0x4E], [0x96, 0xE6, 0x8C]),
            Mode::Alt1 => c([0x3D, 0x24, 0x10], [0xE0, 0x8A, 0x2E], [0xFF, 0xC2, 0x7A]),
            Mode::Alt2 => c([0x2A, 0x10, 0x40], [0xA8, 0x60, 0xD6], [0xDC, 0xAA, 0xFF]),
        }
    }
}

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
        use palette::*;
        Self {
            bg: BG,
            primary: TAN,
            secondary: SLATE,
            accent: TEAL,
            alert: BRICK,
            text: TEXT,
            text_hi: Mode::Primary.colors().label,
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
pub fn contrast(a: Color32, b: Color32) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

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
    // Force dark: the web build otherwise follows the browser's light/dark preference.
    ctx.set_theme(egui::ThemePreference::Dark);
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

// ---------------------------------------------------------------- mode buttons & lists
#[derive(Clone, Copy, Default)]
struct Pending {
    clicks: u32,
    last: f64,
    committed: bool,
}

/// Shared click-count state machine for buttons and list rows.
/// * Pointer: egui's `clicked` / `double_clicked` / `triple_clicked` raise the pending count
///   (1/2/3). The gesture is committed only after the double-click window
///   (`InputOptions::max_double_click_delay`, 0.3 s default) passes without another click, or
///   immediately on a triple click, so a triple click never also fires single/double actions.
///   Clicks after a committed triple within the window are absorbed (clamp, no 4th mode).
/// * Keyboard / accessibility: Shift+click or Shift+Enter = Alt1, Ctrl/Cmd+click or
///   Ctrl/Cmd+Enter = Alt2 (both commit immediately); plain Enter/Space = Primary.
///   A context menu (right-click / long-press) offers every mode by name.
pub fn handle_mode(ui: &Ui, resp: &Response, mode: &mut Mode) -> bool {
    let id = resp.id.with("lcars_mode");
    let (now, mods) = ui.input(|i| (i.time, i.modifiers));
    let delay = ui.ctx().options(|o| o.input_options.max_double_click_delay);
    let mut p: Pending = ui.data(|d| d.get_temp(id)).unwrap_or_default();
    let before = *mode;
    if resp.clicked() {
        if mods.shift || mods.command {
            *mode = mode.after_gesture(if mods.command { 3 } else { 2 });
            p = Pending {
                clicks: 0,
                last: now,
                committed: true,
            };
        } else if p.committed && p.clicks == 3 && now - p.last < delay {
            p.last = now; // absorb 4th+ click of a gesture
        } else {
            let n = if resp.triple_clicked() {
                3
            } else if resp.double_clicked() {
                2
            } else {
                1
            };
            p = Pending {
                clicks: n,
                last: now,
                committed: false,
            };
        }
    }
    if !p.committed && p.clicks > 0 {
        if p.clicks >= 3 || now - p.last >= delay {
            *mode = mode.after_gesture(p.clicks);
            p.committed = true;
        } else {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_secs_f64(delay));
        }
    }
    resp.context_menu(|ui| {
        for m in Mode::ALL {
            if ui.selectable_label(*mode == m, m.name()).clicked() {
                *mode = m;
                ui.close();
            }
        }
    });
    ui.data_mut(|d| d.insert_temp(id, p));
    *mode != before
}

/// Paint an IMG1-style surface: state-tinted dark fill, 1px state border, state label,
/// plus a small mode tag (non-color indicator). Inactive list rows use `flat` (no border).
fn paint_mode_surface(
    ui: &Ui,
    t: &LcarsTheme,
    rect: Rect,
    mode: Mode,
    label: &str,
    hovered: bool,
    row: bool,
) {
    let c = mode.colors();
    let p = ui.painter();
    let (flat, center) = (row, !row);
    let r = CornerRadius::same(4);
    let fill = if flat && mode == Mode::Inactive {
        palette::SURFACE
    } else {
        c.fill
    };
    let fill = if hovered { tint(fill, 0.08) } else { fill };
    p.rect_filled(rect, r, fill);
    if !(flat && mode == Mode::Inactive) || hovered {
        p.rect_stroke(
            rect,
            r,
            Stroke::new(1.0, c.border),
            egui::StrokeKind::Inside,
        );
    }
    let label_col = if flat && mode == Mode::Inactive {
        palette::TEXT_MUTED
    } else {
        c.label
    };
    let font = FontId::monospace(t.data_size);
    if center {
        p.text(
            rect.center(),
            Align2::CENTER_CENTER,
            label.to_uppercase(),
            font.clone(),
            label_col,
        );
    } else {
        p.text(
            pos2(rect.left() + 8.0, rect.center().y),
            Align2::LEFT_CENTER,
            label,
            font.clone(),
            label_col,
        );
    }
    // Pips are painted, not text: U+2056 ⁖ is missing from Antonio, Ubuntu-Light and Hack,
    // and U+2236 ∶ is only in Hack (see AGENTS.md P2).
    paint_pips(
        ui,
        pos2(rect.right() - 10.0, rect.center().y),
        t.data_size,
        mode.pips(),
        c.label,
    );
}

/// Paint 0-3 mode pips centred at `center`, sized from the text height `text_h`,
/// in the arrangements of · (1), ∶ (2, stacked vertically) and ⁖ (3, triangle: one on top, two below).
pub fn paint_pips(ui: &Ui, center: egui::Pos2, text_h: f32, n: u8, color: Color32) {
    let r = (text_h * 0.13).max(1.5);
    let d = r * 2.6; // centre-to-centre spacing
    let offsets: &[Vec2] = match n {
        1 => &[Vec2::ZERO],
        2 => &[vec2(0.0, -d / 2.0), vec2(0.0, d / 2.0)],
        3 => &[
            vec2(0.0, -d / 2.0),
            vec2(-d / 2.0, d / 2.0),
            vec2(d / 2.0, d / 2.0),
        ],
        _ => &[],
    };
    for o in offsets {
        ui.painter().circle_filled(center + *o, r, color);
    }
}

/// Accessible label, e.g. "Scan, mode Alt 1".
fn a11y(resp: &Response, ui: &Ui, label: &str, mode: Mode) {
    let text = format!("{label}, mode {}", mode.short_name());
    let enabled = ui.is_enabled();
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, &text));
}

/// [PROJECT] mode button (IMG1 style). Returns the response; `mode` is updated in place.
pub fn mode_button(
    ui: &mut Ui,
    t: &LcarsTheme,
    label: &str,
    mode: &mut Mode,
    size: Vec2,
) -> Response {
    let (rect, mut resp) = ui.allocate_exact_size(size, Sense::click());
    if handle_mode(ui, &resp, mode) {
        resp.mark_changed();
    }
    a11y(&resp, ui, label, *mode);
    paint_mode_surface(ui, t, rect, *mode, label, resp.hovered(), false);
    if resp.has_focus() {
        ui.painter().rect_stroke(
            rect.expand(2.0),
            CornerRadius::same(5),
            Stroke::new(1.0, t.text),
            egui::StrokeKind::Outside,
        );
    }
    resp
}

/// [PROJECT] list with rows that behave like mode buttons (IMG3 style).
pub fn mode_list(
    ui: &mut Ui,
    t: &LcarsTheme,
    id_salt: &str,
    rows: &mut [(String, Mode)],
    row_h: f32,
) {
    ui.push_id(id_salt, |ui| {
        ui.spacing_mut().item_spacing.y = 2.0;
        for (i, (label, mode)) in rows.iter_mut().enumerate() {
            ui.push_id(i, |ui| {
                let (rect, resp) =
                    ui.allocate_exact_size(vec2(ui.available_width(), row_h), Sense::click());
                handle_mode(ui, &resp, mode);
                a11y(&resp, ui, label, *mode);
                paint_mode_surface(ui, t, rect, *mode, label, resp.hovered(), true);
                if resp.has_focus() {
                    ui.painter().rect_stroke(
                        rect,
                        CornerRadius::same(4),
                        Stroke::new(1.0, t.text),
                        egui::StrokeKind::Outside,
                    );
                }
            });
        }
    });
}

// ---------------------------------------------------------------- readouts
pub fn readout(ui: &mut Ui, t: &LcarsTheme, value: &str, highlight: bool) {
    let c = if highlight { t.text_hi } else { t.text };
    ui.label(egui::RichText::new(value).color(c).size(t.data_size));
}

// ---------------------------------------------------------------- demo
/// Demo state (kept by the caller so modes persist between frames).
pub struct DemoState {
    pub buttons: [(String, Mode); 4],
    pub rows: Vec<(String, Mode)>,
}

impl Default for DemoState {
    fn default() -> Self {
        let b = |l: &str, m| (l.to_owned(), m);
        Self {
            buttons: [
                b("Load", Mode::Inactive),
                b("Dwell", Mode::Primary),
                b("Save", Mode::Alt1),
                b("Scan", Mode::Alt2),
            ],
            rows: [
                ("1.024 West Lafayette", Mode::Inactive),
                ("1.025 Geetingsville", Mode::Primary),
                ("1.026 Covington", Mode::Inactive),
                ("1.027 Newport", Mode::Alt1),
                ("1.028 Marshall", Mode::Inactive),
                ("1.031 Dalton", Mode::Alt2),
                ("1.032 Hortonville", Mode::Inactive),
            ]
            .into_iter()
            .map(|(l, m)| (l.to_owned(), m))
            .collect(),
        }
    }
}

pub fn demo(ui: &mut Ui, t: &LcarsTheme, st: &mut DemoState) {
    use palette::*;
    egui::Panel::top("lcars_header")
        .exact_size(t.bar_thin + t.main_gap)
        .frame(egui::Frame::new().fill(t.bg).inner_margin(t.frame_gap))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                lcars_button(ui, t, "Tune", TAN, vec2(120.0, t.bar_thin), Cap::Left, true);
                for (l, c) in [
                    ("Plan", SLATE),
                    ("Channels", OLIVE),
                    ("Meters", BRICK),
                    ("Levels", TEAL),
                    ("Config", PLUM),
                ] {
                    lcars_button(ui, t, l, c, vec2(110.0, t.bar_thin), Cap::None, true);
                }
                ui.add_space(t.main_gap);
                ui.label(
                    egui::RichText::new("RUSTCARS")
                        .size(t.title_size)
                        .color(t.text),
                );
            });
        });
    egui::CentralPanel::default()
        .frame(egui::Frame::new().fill(t.bg).inner_margin(t.frame_gap))
        .show(ui, |ui| {
            let full = ui.available_rect_before_wrap();
            let split = full.left() + full.width() * 0.55;
            let left_r =
                Rect::from_min_max(full.min, pos2(split - t.main_gap / 2.0, full.bottom()));
            let right_r = Rect::from_min_max(pos2(split + t.main_gap / 2.0, full.top()), full.max);

            ui.scope_builder(UiBuilder::new().max_rect(left_r), |ui| {
                let mut f = LcarsFrame::new(t, Side::Left, 1.0);
                f.top_color = TAN;
                f.bottom_color = SLATE;
                f.min_height = left_r.height();
                let (_, fr) = f.show(ui, t, |ui| {
                    ui.label(egui::RichText::new("MODE BUTTONS").size(t.header_size));
                    ui.horizontal(|ui| {
                        for (l, m) in st.buttons.iter_mut() {
                            mode_button(ui, t, l, m, vec2(96.0, 56.0));
                        }
                    });
                    ui.add_space(t.main_gap);
                    ui.label(
                        egui::RichText::new("CLICK SELECT  ·  DOUBLE ALT1  ·  TRIPLE ALT2")
                            .size(t.data_size),
                    );
                    ui.label(
                        egui::RichText::new("SHIFT/CTRL+CLICK OR RIGHT-CLICK MENU")
                            .size(t.data_size),
                    );
                    ui.add_space(t.main_gap);
                    ui.label(egui::RichText::new("TELEMETRY").size(t.header_size));
                    egui::Grid::new("nums")
                        .spacing(vec2(t.main_gap, t.frame_gap))
                        .show(ui, |ui| {
                            for row in 0..4u64 {
                                for col in 0..4u64 {
                                    let v = (row * 7919 + col * 104729) % 100000;
                                    readout(ui, t, &format!("{v:05}"), row == 1 && col == 2);
                                }
                                ui.end_row();
                            }
                        });
                });
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
                        TEAL,
                        vec2(t.bar_thick, 44.0),
                        Cap::None,
                        true,
                    );
                    lcars_button(
                        ui,
                        t,
                        "Shields",
                        OLIVE,
                        vec2(t.bar_thick, 44.0),
                        Cap::None,
                        true,
                    );
                    lcars_button(
                        ui,
                        t,
                        "Off",
                        OLIVE,
                        vec2(t.bar_thick, 44.0),
                        Cap::None,
                        false,
                    );
                });
            });

            ui.scope_builder(UiBuilder::new().max_rect(right_r), |ui| {
                let mut f = LcarsFrame::new(t, Side::Right, 0.6);
                f.top_color = PLUM;
                f.bottom_color = BRICK;
                f.min_height = right_r.height();
                f.show(ui, t, |ui| {
                    ui.label(egui::RichText::new("CHANNELS").size(t.header_size));
                    mode_list(ui, t, "channels", &mut st.rows, 28.0);
                });
            });
        });
}

/// The demo as an `eframe::App`, shared by the desktop binary and the web (wasm) build.
pub struct DemoApp {
    pub theme: LcarsTheme,
    pub state: DemoState,
}

impl DemoApp {
    pub fn new(ctx: &Context) -> Self {
        let theme = LcarsTheme::default();
        install_lcars_font(ctx);
        apply_theme(ctx, &theme);
        Self {
            theme,
            state: DemoState::default(),
        }
    }
}

impl eframe::App for DemoApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        demo(ui, &self.theme, &mut self.state);
    }
}
