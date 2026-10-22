# AGENTS.md: LCARS UI rules for Grok Build (rustcars, egui / Rust)

This repo (`rustcars`) is the reference implementation of these rules. Library: `src/lib.rs`. Demo binary: `src/main.rs` (`cargo run`). Preview/snapshot test: `tests/preview.rs`. Fonts: `assets/fonts/`.
Before you commit, check that `cargo clippy --all-targets -- -D warnings` and `cargo test` both pass.

## Sources & attribution
- **[SRC]** "LCARS GuideLine" / "Creating a Coherent LCARS Interface" — http://www.lcars-terminal.de/tutorial/guideline.htm
- **[MANIFESTO]** "The LCARS Manifesto" — http://www.lcars-terminal.de/tutorial/manifesto.htm
- Both texts are by **Bracer Jack** (originally on bracercom.com) and were republished from an archive by lcars-terminal.de, which notes that some minor parts are missing.
- LCARS was designed by **Michael Okuda** for *Star Trek*, with the minimalism coming from a request by Gene Roddenberry. Star Trek and LCARS belong to their respective trademark/copyright holders. This file is an unaffiliated design guideline.

**Tags:** [SRC] = guideline page. [MANIFESTO] = manifesto. [SUGGESTION] = added by us (defaults and egui implementation), not from either source.
**Precedence:** when the sources disagree (e.g. gradients), apply the stricter [SRC] rule by default.
**egui version:** the snippets compile against **egui 0.37.0** and egui_kittest 0.37.0, using Rust 1.99.1 stable via rustup and edition 2024. API notes:
- `CornerRadius` (u8 fields) replaced `Rounding` in 0.31.
- `FontData` must be wrapped in `Arc` (0.29+).
- Use `Frame::new()`, not `Frame::none()`.
- In 0.37, `Context::style_mut` became `global_style_mut`.
- `TopBottomPanel`/`SidePanel` are replaced by `egui::Panel::top/left/right(..).exact_size(..)`, and panels are shown with `.show(ui, ..)` on a parent `Ui`.
- `Margin` fields are `i8`, so they can't hold margins wider than 127 px. Use `scope_builder` with a shrunk `max_rect` for wide LCARS bars.

---

## 1. Philosophy
- [MANIFESTO] "The Perfect LCARS System is Invisible." LCARS for real applications puts function first. Decoration must never get in the way of operation.
- [MANIFESTO] The **swept elbow IS LCARS**. It is the single defining shape, with space efficiency as its flaw.
- [MANIFESTO] Simplicity, after Roddenberry's request for panels without much activity. Show nothing useless. Never ask the user to do what the computer can do itself, but don't take away control just to *look* simple.
- [MANIFESTO] Its ideal end state is so minimal that it doesn't look like LCARS, e.g. a blank screen with two buttons.
- [SRC] Clean, flat "vector" look with no gradients or emboss. The frame bends to wrap content.
- [MANIFESTO] Unlike [SRC], gradients and other shading are allowed "as long as it doesn't become a visual distraction". Grok Build should stay flat unless the user asks otherwise.

## 2. Frame & elbow
- [SRC] A frame is one continuous shape. Segment it according to how many options you show up front.
- [SRC][MANIFESTO] **Frame thickness ALWAYS changes at a turn.** The manifesto calls this "OMEGA" and the second most common crime after color misuse.
- [SRC] Do not disfigure the swept elbow (no non-uniform scaling or skew).
- [MANIFESTO] Frames twist into **local frames that group related controls**.
- [MANIFESTO] Make the frame itself useful: break the frame into buttons rather than adding separate buttons, unless a segment is too thin to press. A frame that only labels a category is "already a 20% sin". An empty frame with lots of blank space is fine if the frame itself does work.
- [SUGGESTION] Geometry defaults:
  - Sidebar `bar_thick = 120 px` and horizontal bar `bar_thin = 40 px`, a 3:1 ratio. Keep the ratio between 2:1 and 4:1.
  - Outer elbow radius ≈ `bar_thick`. Inner (concave) radius ≈ `bar_thin`, giving R_out:R_in ≈ 3:1.
  - Gap between frame segments = `frame_gap`.
  - Derive everything from these values and never hardcode per screen.

## 3. Caps & buttons
- [SRC] The rounded cap is the "full stop" that ends a bar. Unconnected capped segments are natural buttons. Odd cap shapes are amateurish.
- [MANIFESTO] The canonical button is the **rounded-rectangle (pill)**. If a label is too long and stretching would break the "LCARS ratio", switch to a variant that fits the surrounding elements, such as a one-side cap or a square segment. The variant images are missing from the archive.
- [MANIFESTO] **Conformity:** buttons in the same frame look alike. Different frames may use different variants.
- [SUGGESTION] Cap radius is exactly half the bar height (a semicircle). The pill aspect ratio is about 2:1 to 4:1. Beyond that, use a one-side cap.
- [SUGGESTION] Labels are uppercase, right- and bottom-aligned, in black on the colored fill. This is a common LCARS convention but is not stated in either source.

## 4. Spacing
- [SRC] Use an invisible grid and **exactly two spacing constants**: Main Frame Spacing and Frame (segment) Spacing. The only exceptions are the four top main buttons and text.
- [SUGGESTION] `frame_gap = 4 px`, `main_gap = 16 px`, applied through `Style::spacing` and `Frame::inner_margin` only.

## 5. Typography
- [SRC] One font, used consistently. **Three sizes:** Main Title, Sub Header (also used for the four top buttons), and Normal Data. **Two text colors:** normal and highlight, where highlight marks data being processed.
- [SUGGESTION] Font: **Antonio** by Vernon Adams, © 2013 The Antonio Project Authors. It is licensed under the **SIL Open Font License 1.1** with no Reserved Font Name, as verified from https://github.com/google/fonts/tree/main/ofl/antonio (OFL.txt) and https://github.com/googlefonts/antonioFont.
  - Bundled in this repo as `assets/fonts/Antonio-Variable.ttf`, the variable font with a default weight of 400.
  - It is loaded via `include_bytes!` in `install_lcars_font()`, wrapped in `Arc<FontData>` and inserted first in `FontFamily::Proportional`. egui's default fonts stay as fallbacks.
- **Rule (license):** whenever a font file is bundled, keep `OFL.txt` and a notice file (`FONTS.md`: copyright, source URL, license) in the same directory and ship them with every distribution.
  - Never sell the font file on its own.
  - If you modify the font, it must stay under the OFL.
  - Do not bundle any font whose license you haven't confirmed (e.g. Swiss 911 or fan-made "LCARS" fonts).
- [SUGGESTION] Sizes are 36 / 22 / 15 px for title / sub header / data.

## 6. Color
- [SRC] There is no official palette. Use color theory. Recommended maximum by number of colors:
  - **1 color:** at most 5 tints/shades total.
  - **2 colors:** at most 5 combinations total.
  - **3 colors:** good; stop here.
  - **4 colors:** each needs a meaning (idle, normal, real-time, text).
  - **5 colors:** must justify the 5th, e.g. what does flashing in it mean.
  - **6 or more:** avoid.
- [SRC] Monotone is better than unrelated multicolor.
- [MANIFESTO] Defers to [SRC] for color. Allows shading when it isn't distracting.
- [SUGGESTION] Default palette (fan-common values, not from the sources). Background is black `#000000`. Contrast is measured against black.

| Role | Hex | egui | Contrast on black |
|---|---|---|---|
| primary / normal ops, text | `#FF9900` | `Color32::from_rgb(0xFF,0x99,0x00)` | 9.8:1 |
| secondary / idle | `#CC99CC` | `Color32::from_rgb(0xCC,0x99,0xCC)` | 9.0:1 |
| accent / real-time, selected | `#9999FF` | `Color32::from_rgb(0x99,0x99,0xFF)` | 8.4:1 |
| alert (5th, meaning = alert) | `#CC4444` | `Color32::from_rgb(0xCC,0x44,0x44)` | 4.5:1 |
| text highlight (tint of primary) | `#FFCC99` | `Color32::from_rgb(0xFF,0xCC,0x99)` | 14.3:1 |

- [SUGGESTION] **State colors** are derived as tints/shades of the base so no new hues are added:
  - hover: +30% toward white
  - pressed: −25% toward black
  - disabled: −60% toward black
  - selected: switch to `accent`
  - alert: `alert` fill, optionally blinking at about 1 Hz
- See `tint()` in the code below.

## 7. Animation & sound
- [SRC] For decorative displays, avoid obvious loops. Give each element its own timeline with a different duration, and generate random numbers in code rather than keyframing them.
- [MANIFESTO] Animation is **not core**. Interfaces must **not rearrange themselves**:
  - A new task gets a **new screen**.
  - New controls must never shift existing buttons. If that can't be avoided, use a new screen.
  - Transitions are a simple, snappy **fade in/out (not crossfade)** and must be **under 1 s**. Even 1 s is "way too long".
  - Decorative "flashies" are fine only if they never get in the way of operation.
- [MANIFESTO] Sound is up to you, but don't make it annoying.
- [SUGGESTION] Timing:
  - Fade out about 120 ms, then fade in about 150 ms, using `ctx.animate_bool_with_time`.
  - Readouts tick every 200–400 ms. Use a different period per element, coprime periods such as 230/310/470 ms, plus a `fastrand` RNG.
  - Use `ctx.request_repaint_after(...)` instead of continuous repaint.
  - Alert blink about 1 Hz. Never use frequencies between 3 and 50 Hz (photosensitivity, WCAG 2.3.1).
  - Respect a "reduce motion" setting by turning decoration off.

## 8. Accessibility [SUGGESTION]
- Text must meet at least 4.5:1 contrast (WCAG AA). All palette fills above pass with black labels on them, but `#CC4444` is borderline, so use larger or bolder text on alert.
- Don't rely on color alone: alert states also need a label change ("ALERT") or blinking.
- Minimum hit target is about 44×44 px, which matches the manifesto's "too thin for fingers" rule.
- Make every button reachable by keyboard. egui gives focus to widgets with `Sense::click()`, so draw a visible focus indicator using `response.has_focus()`.

## 9. Code (compile-checked, egui 0.32.3) [SUGGESTION]
Covers: theme, `ink_on` (automatic black/white text by WCAG luminance), font loading, Visuals, pill buttons, `Elbow`/`elbow_shapes`/`paint_elbow` for all four corners, `LcarsFrame` (paint-behind chrome sized to its content), readouts, the demo, and the kittest preview test.

```rust
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

// ---- tests/preview.rs ----
//! Renders the demo with egui_kittest + wgpu (software Vulkan/lavapipe works headless).
//! `cargo test --test preview -- --nocapture` writes docs/preview.png
use egui_kittest::Harness;

#[test]
fn render_preview() {
    let t = rustcars::LcarsTheme::default();
    let mut setup = true;
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1280.0, 800.0))
        .wgpu()
        .build_ui(move |ui| {
            if setup {
                rustcars::install_lcars_font(ui.ctx());
                rustcars::apply_theme(ui.ctx(), &t);
                setup = false;
            }
            rustcars::demo(ui, &t);
        });
    harness.run_steps(4);
    let mut img = harness.render().expect("wgpu render failed");
    // build_ui leaves a transparent margin around the root Ui; flatten onto LCARS black.
    for px in img.pixels_mut() {
        let a = px[3] as u32;
        for k in 0..3 {
            px[k] = (px[k] as u32 * a / 255) as u8;
        }
        px[3] = 255;
    }
    img.save(concat!(env!("CARGO_MANIFEST_DIR"), "/docs/preview.png"))
        .unwrap();
    // Regression snapshot (tests/snapshots/lcars_demo.png); UPDATE_SNAPSHOTS=1 to refresh.
    harness.snapshot("lcars_demo");
}
```
Notes:
- Call `install_lcars_font(ctx)` once at startup, before `apply_theme`.
- `Elbow::new(Corner::{TL,TR,BL,BR}, thick, thin, color, bg).paint(ui, rect)` (or `elbow_shapes(rect, &elbow)` for paint-behind) defines the geometry once and mirrors it for all four corners. It debug-asserts that `thick != thin`. The swept block is sized to at least `2*r_out` because epaint clamps corner radii to half the rect's shorter side.
- End each elbow's free bar with a separate rounded cap segment placed `frame_gap` away (see `elbow_with_cap` in `demo`).
- **Paint-behind frames:** `LcarsFrame::show` first reserves `painter.add(Shape::Noop)`, then lays out the content and finally calls `painter.set(idx, Shape::Vec(chrome))`. The elbows, bars and caps are therefore sized to the real content rect and drawn behind the widgets. This technique is inspired by paxel/dedup-rs (MIT); the code is our own.
- **Preview:** run `cargo test --test preview` in the repo root.
  - It uses `egui_kittest` with the `wgpu` and `snapshot` features. On machines without a GPU, Mesa lavapipe (`mesa-vulkan-drivers`) provides software Vulkan.
  - It writes `docs/preview.png`, plus a regression snapshot at `tests/snapshots/lcars_demo.png`. Set `UPDATE_SNAPSHOTS=1` to refresh the snapshot.
- **Numeric blocks:** use `egui::Grid` with right-aligned `readout` cells. Each column ticks on its own period.
- **Layout recipe:**
  - `TopBottomPanel::top` holds the header bar (thin) and the title.
  - `SidePanel::left` (`exact_width(bar_thick)`, not resizable) holds stacked frame-segment buttons separated by `frame_gap`.
  - `CentralPanel` uses `inner_margin(main_gap)`.
  - Paint the elbow where the side and top bars meet.
  - Optionally add a `TopBottomPanel::bottom` mirror elbow.

## 10. Agent checklist (Grok Build)
**Do**
- [ ] Use one `LcarsTheme` as the source of all colors, sizes and gaps. No inline literals.
- [ ] Vary thickness at every elbow, and draw elbows from primitives proportional to thickness.
- [ ] Use semicircular caps, with pill buttons that look alike within each frame.
- [ ] Turn frame segments into buttons instead of adding extra ones.
- [ ] Keep to two spacing values, one font, three sizes, two text colors, at most 5 colors, and give meaning to the 4th and 5th.
- [ ] Open a new screen for a new task. Never shift existing controls.
- [ ] Keep transitions as fades under 1 s (aim for under 250 ms). Drive decorative animation from code on independent periods.
- [ ] Meet contrast and hit-target minimums. Don't encode meaning with color alone.

**Don't**
- [ ] Use the same thickness across a turn, stretch elbows, or use odd cap shapes.
- [ ] Use shadows, emboss, or distracting gradients.
- [ ] Use unrelated hues, more than 5 colors, or extra fonts or sizes.
- [ ] Use crossfades, layout-reshuffling animation, or flashing between 3 and 50 Hz.
- [ ] Use decoration that obstructs operation, or frames that only label a category.
- [ ] Add annoying sounds.

## 11. Breaking the rules
- [SRC] Break rules only once you understand why they exist.
- For Grok Build, deviate only when the user asks, and explain the reason in a code comment.

## Resources (verified 2026-10-22)
**Design references**
- https://www.thelcars.com/colors.php: hex codes and CSS variable names for the Classic, Nemesis Blue, Lower Decks, Lower Decks PADD, Picard and Voyager themes. Some colors changed in "LCARS v26". No license stated.
- https://github.com/joernweissenborn/lcars: CSS framework, MIT. Its demo (http://joernweissenborn.github.io/lcars/) shows a full set of components.
- https://github.com/Aricwithana/LCARS-SDK: reference layouts and elements. The license is unclear (GitHub reports NOASSERTION), so look but don't copy.
- https://github.com/louh/lcars: responsive HTML/CSS layout, GPL-3.0. Reference only.
- https://github.com/th3jesta/ha-lcars: Home Assistant theme, MIT.
- https://github.com/tobykurien/rpi_lcars: Pygame touch UI, MIT.
- https://memory-alpha.fandom.com/wiki/LCARS: fan-wiki history. It blocked automated fetching, so its content wasn't reviewed.

**Rust code**
- https://github.com/paxel/dedup-rs (`crates/dedup-gui/src/lcars.rs`): egui 0.35, MIT. Source of the paint-behind elbow idea, the luminance-based ink color and the kittest snapshots.
- https://github.com/DavidMikeSimon/lcars_hass_egui: older egui, no license (don't copy). Its frame code is a simple bar-rectangle layout.
- https://github.com/jtmcg3/lcars_stats: ratatui terminal UI, no license. Useful ideas: a palette constants table and a header/footer/gauge pattern.
- https://github.com/highwaycoder/lcars-rs (crates.io `lcars` 0.1.0): early sketch, no license.

## Out of scope / gaps
- About half the manifesto is about an imagined "LCARS programming language" (auto-imports, avoiding Hungarian notation, one-line APIs, against premature OOP) and future concepts (no keyboards, voice, holography). It isn't UI guidance, so it was left out. If you want to borrow from it, "one-line calls for common actions" fits the API design of the widget helpers.
- Neither source gives hex colors, sizes, radii or font sizes, so every number here is a [SUGGESTION].
- The button-variant and frame example images are missing from the archive.
