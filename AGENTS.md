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

## 0. Project rules [PROJECT] (David Norris; these override any conflicting [SRC]/[MANIFESTO]/[SUGGESTION] rule)
Reference screenshots: IMG1 buttons (LOAD/SAVE/SCAN/DWELL), IMG2 tab palette (Tune/Plan/Channels/Meters/Levels/Config), IMG3 lists. The hex values below were sampled from those PNGs with PIL.

### P1. Buttons (IMG1)
- Clickable buttons that perform a function are **rectangles with a small rounding** (`CornerRadius::same(4)`).
- Each has a **dark fill tinted by its state**, a **1 px border in the state color** (`StrokeKind::Inside`), and an **uppercase monospace label in the state's label color**.
- Implementation: `mode_button()`.
- This replaces the pill-button guidance in §3 for function buttons. The LCARS frame segments and the header tab strip keep the solid muted IMG2 tab style (`lcars_button`), because they are frame chrome and navigation, not multi-mode functions.

### P1b. Toggle buttons
- A button that **switches a state on and off** (Live, Data, Follow, Mute, Run…) is a toggle. Toggles get a **thin border** (`TOGGLE_BORDER` = 1.5 px, `StrokeKind::Inside`) so they read differently from momentary buttons.
- **Momentary buttons** (◄ DAY, TODAY, −, +, FIT…) do something once and keep no state. They stay **solid filled with no outline**.
- **On:** solid fill in the toggle's hue, border a light tint of the hue (or the ink colour if the tint is under 3:1). **Off:** dark page fill (`#1B1B1B`), border in the hue, brightened until it reaches 3:1 against the fill.
- **State in text:** the label starts with **● when on and ○ when off** (or reads ON/OFF). Never show the state by colour or border alone. The marker is painted (`paint_toggle_marker_label`), because Antonio has no ●/○ glyphs.
- Implementation: `toggle_button()`, `toggle_colors()`, `paint_toggle_marker_label()`, `toggle_label()`. Checked by `tests/toggle.rs`.
- A toggle with click modes stays a `mode_button`; this rule covers plain on/off buttons.

### P1c. Detachable elbows
- A group box (`group::GroupBox`, thin titled elbow) may be `.detachable(name)`. Its **end pill is the detach/attach toggle**: it keeps the elbow-end shape (round right end), gets a `PILL_BORDER` (1.5 px) outline, and reads **DETACH** or **ATTACH** (state in text, with a tooltip).
- Detached, the elbow and everything in it go to their own window (immediate viewport, embedded as an egui window where the backend has no viewports). The parent keeps the elbow with a plain-text stub and an ATTACH pill. Closing the window attaches it.
- Contents run exactly once per frame. Nesting works: detached children of a detached parent keep their own windows.
- Implementation: `src/group.rs` (`GroupBox`, `DetachState`, `detach_pill`, `detached_window`). Checked by `tests/group.rs` with the real font and theme.

### P2. Click-count modes
- One control, one function, up to three operating modes:

| Gesture | Mode | Color | Pips |
|---|---|---|---|
| none | Inactive | gray | – |
| single click | Primary selection | green | · (1) |
| double click | Alternate 1 | orange | ∶ (2, stacked) |
| triple click | Alternate 2 | purple | ⁖ (3, triangle) |

- **Never more than triple.** `Mode::from_clicks(n)` maps n ≥ 3 to Alt2. Extra clicks in the same gesture (4th, 5th, …) are absorbed and don't cycle.
- **Deactivating:** repeating the gesture of the current mode turns the control off (Inactive). A different gesture switches directly to that mode. For example, if a button is in Alt1, a double click turns it off and a single click switches it to Primary. Implementation: `Mode::after_gesture`.
- **No double-firing:**
  - egui's `clicked`/`double_clicked`/`triple_clicked` only raise a pending click count.
  - The gesture is **committed once**: after `InputOptions::max_double_click_delay` (0.3 s default) passes with no further click, or immediately on a triple click.
  - So a triple click never also triggers the single or double action. The cost is that single and double clicks take effect after a 0.3 s delay. Implementation: `handle_mode`.
- **Keyboard and accessibility:**
  - Enter/Space on a focused control = single click.
  - Shift+click or Shift+Enter = Alt1, applied immediately.
  - Ctrl/Cmd+click or Ctrl/Cmd+Enter = Alt2, applied immediately.
  - Right-click (or long-press) opens a context menu listing every mode by name, including Off.
  - Focused controls get a visible focus ring.
- **Never rely on color alone:** every active mode also shows **mode pips**: 1 dot (·), 2 dots stacked vertically (∶) or 3 dots in a triangle (⁖). Inactive shows none.
  - **Placement:** right side of the button or row, vertically centred, 10 px from the right edge.
  - **Size:** radius = 0.13 × text height (min 1.5 px), spacing 2.6 × radius.
  - **Color:** the state's label color, which reaches ≥ 3:1 against the fill (SC 1.4.11). The pairs are 8.49–13.36:1 (see the table).
  - **Painted, not text:** pips use `Painter::circle_filled` (`paint_pips`). We checked the Unicode characters U+00B7 ·, U+2236 ∶ and U+2056 ⁖ with fontTools against the fonts actually in use (Antonio, and egui's built-in Ubuntu-Light, Hack, NotoEmoji and emoji-icon fonts):
    - · is in Antonio, Ubuntu-Light and Hack.
    - ∶ is only in Hack.
    - ⁖ is in **none** of them.
    - As text, ⁖ would show as a missing-glyph box, and the pips would look different from font to font. Dice faces ⚀⚁⚂ were also considered and rejected for the same coverage reason, and because they read as decoration.
  - **Screen readers:** the mode name is exposed through AccessKit widget info, e.g. "Scan, mode Alt 1". Mode names are also listed in the context menu.

### P3. Palette (IMG2) and WCAG 2.2 AA
- **Sampled colors:**
  - Backgrounds: page `#1B1B1B`, list row/surface `#202020`, tab strip `#303030`.
  - Tabs: Tune `#9C805A`, Plan `#3A4E66`, Channels `#56603A`, Meters `#70463E`, Levels `#345A54`, Config `#604666`.
  - IMG1 states:
    - Inactive: fill `#1C1C20`, border `#4E4E54`, label `#E8E4D8`.
    - Purple: fill `#2A1040`, border `#A860D6`, label `#DCAAFF`.
    - Green: fill `#123A1C`, border `#2EB04E`, label `#96E68C`.
  - IMG3: selected row fill `#2A1040` with a brighter purple border `#BA78E6`; green table row `#2D4735`; row text `#8B8B8B`.
- **Adjustments for AA:**
  - Tab hues below 3:1 against the background were brightened in the same hue for frame bars and segments (SC 1.4.11), e.g. Plan `#3A4E66` became `#516D8F`. Tan is unchanged.
  - The inactive border `#4E4E54` (2.0:1) was raised to `#7A7A82`.
  - Muted row text `#8B8B8B` was raised to `#9A9A9A` for margin.
  - The orange state was added to fit the palette: fill `#3D2410`, border `#E08A2E`, label `#FFC27A`.
- **Thresholds:** text ≥ 4.5:1 (all text, including 12–15 px labels); large text ≥ 3:1; borders and non-text UI ≥ 3:1. Labels on frame segments use `ink_on()`, which picks black or white for the highest contrast.
- **Verified by `tests/modes.rs::wcag_contrast`.** Run `cargo test --test modes -- --nocapture` to print this table:

| Pair | Colors | Ratio | Minimum |
|---|---|---|---|
| Inactive label (text) | #e8e4d8ff on #1c1c20ff | 13.36:1 | 4.5:1 |
| Inactive pips vs fill (1.4.11) | #e8e4d8ff on #1c1c20ff | 13.36:1 | 3:1 |
| Inactive border vs fill (1.4.11) | #7a7a82ff on #1c1c20ff | 3.99:1 | 3:1 |
| Inactive border vs BG (1.4.11) | #7a7a82ff on #1b1b1bff | 4.05:1 | 3:1 |
| Inactive border vs SURFACE (1.4.11) | #7a7a82ff on #202020ff | 3.83:1 | 3:1 |
| Primary label (text) | #96e68cff on #123a1cff | 8.49:1 | 4.5:1 |
| Primary pips vs fill (1.4.11) | #96e68cff on #123a1cff | 8.49:1 | 3:1 |
| Primary border vs fill (1.4.11) | #2eb04eff on #123a1cff | 4.51:1 | 3:1 |
| Primary border vs BG (1.4.11) | #2eb04eff on #1b1b1bff | 6.10:1 | 3:1 |
| Primary border vs SURFACE (1.4.11) | #2eb04eff on #202020ff | 5.77:1 | 3:1 |
| Alt1 label (text) | #ffc27aff on #3d2410ff | 9.09:1 | 4.5:1 |
| Alt1 pips vs fill (1.4.11) | #ffc27aff on #3d2410ff | 9.09:1 | 3:1 |
| Alt1 border vs fill (1.4.11) | #e08a2eff on #3d2410ff | 5.38:1 | 3:1 |
| Alt1 border vs BG (1.4.11) | #e08a2eff on #1b1b1bff | 6.43:1 | 3:1 |
| Alt1 border vs SURFACE (1.4.11) | #e08a2eff on #202020ff | 6.09:1 | 3:1 |
| Alt2 label (text) | #dcaaffff on #2a1040ff | 9.01:1 | 4.5:1 |
| Alt2 pips vs fill (1.4.11) | #dcaaffff on #2a1040ff | 9.01:1 | 3:1 |
| Alt2 border vs fill (1.4.11) | #a860d6ff on #2a1040ff | 4.26:1 | 3:1 |
| Alt2 border vs BG (1.4.11) | #a860d6ff on #1b1b1bff | 4.35:1 | 3:1 |
| Alt2 border vs SURFACE (1.4.11) | #a860d6ff on #202020ff | 4.12:1 | 3:1 |
| Text on BG | #d9c7a8ff on #1b1b1bff | 10.40:1 | 4.5:1 |
| Text on SURFACE | #d9c7a8ff on #202020ff | 9.84:1 | 4.5:1 |
| Muted row text on SURFACE | #9a9a9aff on #202020ff | 5.79:1 | 4.5:1 |
| Tan frame/segment vs BG (1.4.11) | #9c805aff on #1b1b1bff | 4.63:1 | 3:1 |
| Tan segment label (auto ink, large 22px) | #000000ff on #9c805aff | 5.65:1 | 4.5:1 |
| Slate frame/segment vs BG (1.4.11) | #516d8fff on #1b1b1bff | 3.23:1 | 3:1 |
| Slate segment label (auto ink, large 22px) | #ffffffff on #516d8fff | 5.34:1 | 4.5:1 |
| Olive frame/segment vs BG (1.4.11) | #646f43ff on #1b1b1bff | 3.20:1 | 3:1 |
| Olive segment label (auto ink, large 22px) | #ffffffff on #646f43ff | 5.39:1 | 4.5:1 |
| Brick frame/segment vs BG (1.4.11) | #945c52ff on #1b1b1bff | 3.21:1 | 3:1 |
| Brick segment label (auto ink, large 22px) | #ffffffff on #945c52ff | 5.37:1 | 4.5:1 |
| Teal frame/segment vs BG (1.4.11) | #43736cff on #1b1b1bff | 3.20:1 | 3:1 |
| Teal segment label (auto ink, large 22px) | #ffffffff on #43736cff | 5.37:1 | 4.5:1 |
| Plum frame/segment vs BG (1.4.11) | #815e89ff on #1b1b1bff | 3.20:1 | 3:1 |
| Plum segment label (auto ink, large 22px) | #ffffffff on #815e89ff | 5.39:1 | 4.5:1 |

- The disabled "OFF" segment is dimmed on purpose. WCAG exempts inactive components from contrast requirements.

### P4. Lists (IMG3)
- List rows behave exactly like mode buttons: same gestures, modes, colors, pips, keyboard shortcuts and context menu. Implementation: `mode_list()`.
- Inactive rows are flat `#202020` with muted text and no border. They show a gray border only on hover.
- Activated rows get the state's tinted fill plus a **thin 1 px border** (like the purple selection in IMG3) and right-aligned mode pips.

### P5. Web interface (optional)
- The web interface is **optional and OFF by default**. Never add `web` or `web-server` to `default` features, and never start the server without an explicit `--enable`.
- The `rustcars-web` server binds **127.0.0.1 by default**. Exposing it on a LAN requires an explicit `--bind`, and it must print the no-authentication warning at startup.
- **Device access (GQRX TCP, USB, …) happens only server-side** through a `DeviceBackend` implementation behind the WebSocket bridge. The wasm UI must never assume it can reach devices directly.
- Shared UI code must compile for both targets. Use egui's input time (`ui.input(|i| i.time)`) or the `web-time` crate, never `std::time::Instant`, which panics on wasm32.
- `apply_theme` forces `ThemePreference::Dark`, because browsers otherwise apply their light/dark preference.
- Before committing, run `scripts/check.sh`. It covers fmt, clippy for the default/wasm/server builds, and the tests.

### P5b. Single-meaning buttons and examples
- For functions with exactly one meaning (digits, operators, Confirm/Cancel), use `action_button`. It has the IMG1 look with no click modes and fires on every click, so fast repeated presses are never merged.
- The reference apps are `examples/calculator`, `examples/diary`, `examples/ham_rf`, `examples/timers`, `examples/hexview` and `examples/smith`. They are desktop only, and their extra dependencies (rusqlite, directories, chrono) are dev-dependencies.
- A unit or direction with 2-3 choices is one `mode_button` showing the current choice (click = first, double = second, triple = third). It always has a value (repeating a gesture must not turn it off) and ignores gestures past the last choice. Name the gestures in a hint line. For 4+ choices use a row of single-choice buttons.
- Custom painted widgets (e.g. the Smith chart) can take click modes too: call `handle_mode` on the widget's own `Response`, and remember the click position, because the gesture commits after the double-click window.
- Keep formulas and parsers in a module with no egui, covered by known-value unit tests. Label approximate data and informational data (cable loss, band privileges) on screen.

### P6. Developer manual
- The human-facing guide is `docs/MANUAL.md` (chapters in `docs/manual/`). The PDF edition is `docs/rustcars-manual.pdf`, rebuilt with `docs/build-manual.sh`.
- Keep the manual's examples consistent with the code: `examples/minimal.rs` must match chapter 2.

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

## 9. Code (compile-checked, egui 0.37) [SUGGESTION]/[PROJECT]
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
            w.corner_radius = CornerRadius::same(4);
        }
        // Text fields (egui's own widgets): visible surface + 1px border >= 3:1 (WCAG 1.4.11).
        v.extreme_bg_color = palette::SURFACE;
        v.widgets.inactive.bg_stroke = Stroke::new(1.0, Mode::Inactive.colors().border);
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, t.text);
        v.selection.stroke = Stroke::new(1.0, t.text);
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
    let p = ui
        .painter()
        .with_clip_rect(rect.expand(1.0).intersect(ui.clip_rect()));
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

/// IMG1-style single-action button (no click modes): fires immediately on every click,
/// so fast repeated presses (e.g. typing digits) are never merged into a double/triple click.
/// Use it for functions with exactly one meaning.
pub fn action_button(ui: &mut Ui, t: &LcarsTheme, label: &str, size: Vec2) -> Response {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    paint_mode_surface(ui, t, rect, Mode::Inactive, label, resp.hovered(), false);
    if resp.is_pointer_button_down_on() {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(4), Color32::from_white_alpha(18));
    }
    if resp.has_focus() {
        ui.painter().rect_stroke(
            rect.expand(2.0),
            CornerRadius::same(5),
            Stroke::new(1.0, t.text),
            egui::StrokeKind::Outside,
        );
    }
    let enabled = ui.is_enabled();
    let l = label.to_owned();
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, &l));
    resp
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

// ---- tests/modes.rs ----
//! [PROJECT] click-count modes and WCAG 2.2 AA contrast checks.
use egui::Color32;
use rustcars::{Mode, contrast, ink_on, palette::*};

#[test]
fn click_count_to_mode() {
    assert_eq!(Mode::from_clicks(0), Mode::Inactive);
    assert_eq!(Mode::from_clicks(1), Mode::Primary);
    assert_eq!(Mode::from_clicks(2), Mode::Alt1);
    assert_eq!(Mode::from_clicks(3), Mode::Alt2);
    for n in 4..10 {
        assert_eq!(
            Mode::from_clicks(n),
            Mode::Alt2,
            "clicks beyond 3 clamp to triple"
        );
    }
}

#[test]
fn gesture_resolution() {
    // gesture selects its mode
    assert_eq!(Mode::Inactive.after_gesture(1), Mode::Primary);
    assert_eq!(Mode::Inactive.after_gesture(2), Mode::Alt1);
    assert_eq!(Mode::Primary.after_gesture(3), Mode::Alt2);
    assert_eq!(Mode::Alt2.after_gesture(1), Mode::Primary);
    // repeating the current mode's gesture turns it off
    assert_eq!(Mode::Primary.after_gesture(1), Mode::Inactive);
    assert_eq!(Mode::Alt1.after_gesture(2), Mode::Inactive);
    assert_eq!(Mode::Alt2.after_gesture(3), Mode::Inactive);
    assert_eq!(Mode::Alt2.after_gesture(5), Mode::Inactive);
    // every mode has a distinct pip count (non-color indicator), never more than 3
    let pips: Vec<u8> = Mode::ALL.iter().map(|m| m.pips()).collect();
    assert_eq!(pips, vec![0, 1, 2, 3]);
}

fn check(name: &str, fg: Color32, bg: Color32, min: f32, out: &mut Vec<String>) {
    let r = contrast(fg, bg);
    out.push(format!(
        "| {name} | {} on {} | {r:.2}:1 | {min}:1 |",
        fg.to_hex(),
        bg.to_hex()
    ));
    assert!(r >= min, "{name}: {r:.2} < {min}");
}

#[test]
fn wcag_contrast() {
    let mut out = vec![];
    for m in Mode::ALL {
        let c = m.colors();
        let n = format!("{m:?}");
        check(&format!("{n} label (text)"), c.label, c.fill, 4.5, &mut out);
        check(
            &format!("{n} pips vs fill (1.4.11)"),
            c.label,
            c.fill,
            3.0,
            &mut out,
        );
        check(
            &format!("{n} border vs fill (1.4.11)"),
            c.border,
            c.fill,
            3.0,
            &mut out,
        );
        check(
            &format!("{n} border vs BG (1.4.11)"),
            c.border,
            BG,
            3.0,
            &mut out,
        );
        check(
            &format!("{n} border vs SURFACE (1.4.11)"),
            c.border,
            SURFACE,
            3.0,
            &mut out,
        );
    }
    check("Text on BG", TEXT, BG, 4.5, &mut out);
    check("Text on SURFACE", TEXT, SURFACE, 4.5, &mut out);
    check(
        "Muted row text on SURFACE",
        TEXT_MUTED,
        SURFACE,
        4.5,
        &mut out,
    );
    for (n, c) in [
        ("Tan", TAN),
        ("Slate", SLATE),
        ("Olive", OLIVE),
        ("Brick", BRICK),
        ("Teal", TEAL),
        ("Plum", PLUM),
    ] {
        check(
            &format!("{n} frame/segment vs BG (1.4.11)"),
            c,
            BG,
            3.0,
            &mut out,
        );
        check(
            &format!("{n} segment label (auto ink, large 22px)"),
            ink_on(c),
            c,
            4.5,
            &mut out,
        );
    }
    println!("{}", out.join("\n"));
}

// ---- tests/preview.rs ----
//! Renders the demo with egui_kittest + wgpu (software Vulkan/lavapipe works headless).
//! `cargo test --test preview -- --nocapture` writes docs/preview.png
use egui_kittest::Harness;

#[test]
fn render_preview() {
    let t = rustcars::LcarsTheme::default();
    let mut st = rustcars::DemoState::default();
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
            rustcars::demo(ui, &t, &mut st);
        });
    harness.run_steps(4);
    let mut img = harness.render().expect("wgpu render failed");
    // build_ui leaves a transparent margin around the root Ui; flatten onto palette::BG (#1B1B1B).
    for px in img.pixels_mut() {
        let a = px[3] as u32;
        for k in 0..3 {
            px[k] = ((px[k] as u32 * a + 0x1B * (255 - a)) / 255) as u8; // onto palette::BG
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
- [ ] [PROJECT] Build function buttons and list rows with `mode_button`/`mode_list`: IMG1 style, gray/green/orange/purple modes, never a 4th mode, painted mode pips plus AccessKit mode names, keyboard and context-menu access.
- [ ] [PROJECT] Draw on/off buttons with `toggle_button` (thin border, ●/○ marker). Momentary buttons stay solid and borderless.
- [ ] [PROJECT] Take colors from `palette::*` and `Mode::colors()`, and keep `tests/modes.rs` passing whenever you change a color.
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
