//! Thin LCARS group box: a thin bar across the top that turns down the left
//! edge through a small rounded elbow, a short closing cap on the right, and
//! the group's name knocked out of the bar in `ink_on` colour.
//!
//! Purely visual. The chrome is painted *behind* the contents (paint-behind
//! with a reserved shape slot), so it is sized to the contents' real height.
//! Reusable on any dense page: `group_box(ui, "Radio", palette::SLATE, |ui| ...)`.
//!
//! Lock safety: no `ctx.data*` / `ctx.input` closures are used here.

use egui::{Align2, Color32, CornerRadius, FontId, Id, Rect, Response, Sense, Shape, Stroke, StrokeKind, Ui, UiBuilder, Vec2, ViewportBuilder, ViewportId, pos2, vec2};
use crate::{Corner, Elbow, elbow_shapes, ink_on};

/// Sizes for the thin elbow. Much thinner than rustcars' `bar_thin` (40).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Metrics {
    /// Height of the top bar (the title lives in it).
    pub bar: f32,
    /// Width of the left rail.
    pub rail: f32,
    /// Outer elbow radius.
    pub r_out: f32,
    /// Inner (concave) elbow radius.
    pub r_in: f32,
    /// Gap between chrome and contents.
    pub gap: f32,
    /// Space under the contents before the next widget.
    pub bottom: f32,
    /// Width of the closing cap at the right end of the bar.
    pub cap_w: f32,
    /// Gap between the bar and the cap.
    pub cap_gap: f32,
    /// Narrower than this, the cap is dropped.
    pub cap_min_width: f32,
    /// Title font size.
    pub title_size: f32,
    /// Space between the elbow's curve and the title.
    pub title_pad: f32,
}

impl Default for Metrics {
    fn default() -> Self {
        Self {
            bar: 16.0,
            rail: 8.0,
            r_out: 12.0,
            r_in: 6.0,
            gap: 6.0,
            bottom: 10.0,
            cap_w: 18.0,
            cap_gap: 4.0,
            cap_min_width: 160.0,
            title_size: 13.0,
            title_pad: 6.0,
        }
    }
}

/// Where every part goes, for a group at (`left`, `top`) of `width` whose
/// contents are `content_h` tall.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    /// Whole group, chrome included.
    pub outer: Rect,
    /// Rect handed to rustcars' `elbow_shapes` (bar + rail + curve).
    pub elbow: Rect,
    /// Horizontal bar (up to the cap gap).
    pub bar: Rect,
    /// Left rail, full height.
    pub rail: Rect,
    /// Closing cap, if wide enough.
    pub cap: Option<Rect>,
    /// Space for the contents.
    pub inner: Rect,
    /// Where the title may be drawn (inside the bar, right of the curve).
    pub title: Rect,
    /// Gap at the right end of the bar for a control (a control), if asked.
    pub slot: Option<Rect>,
}


pub fn layout(left: f32, top: f32, width: f32, content_h: f32, m: &Metrics) -> Layout {
    layout_with_slot(left, top, width, content_h, m, 0.0)
}

/// [`layout`] with a `slot_w` gap at the right end of the bar, before the cap.
/// The bar stops short of it. A group too narrow for title room keeps no slot.
pub fn layout_with_slot(
    left: f32,
    top: f32,
    width: f32,
    content_h: f32,
    m: &Metrics,
    slot_w: f32,
) -> Layout {
    let width = width.max(m.rail + m.gap + 1.0);
    let right = left + width;
    let inner = Rect::from_min_size(
        pos2(left + m.rail + m.gap, top + m.bar + m.gap),
        vec2((width - m.rail - m.gap).max(0.0), content_h.max(0.0)),
    );
    let bottom = inner.bottom() + m.bottom;
    let (bar_end, cap) = if width >= m.cap_min_width {
        let cap = Rect::from_min_max(pos2(right - m.cap_w, top), pos2(right, top + m.bar));
        (cap.left() - m.cap_gap, Some(cap))
    } else {
        (right, None)
    };
    let curve_w = (m.rail + m.r_in).max(2.0 * m.r_out);
    let (bar_end, slot) = if slot_w > 0.0 && bar_end - slot_w - m.cap_gap > left + curve_w + m.title_pad {
        let r = Rect::from_min_max(pos2(bar_end - slot_w, top), pos2(bar_end, top + m.bar));
        (r.left() - m.cap_gap, Some(r))
    } else {
        (bar_end, None)
    };
    let elbow = Rect::from_min_max(pos2(left, top), pos2(bar_end, bottom));
    let bar = Rect::from_min_max(pos2(left, top), pos2(bar_end, top + m.bar));
    let rail = Rect::from_min_max(pos2(left, top), pos2(left + m.rail, bottom));
    // rustcars sizes the swept corner block as max(rail + r_in, 2 * r_out).
    let title_left = (left + curve_w + m.title_pad).min(bar_end);
    let title = Rect::from_min_max(
        pos2(title_left, top),
        pos2((bar_end - m.title_pad).max(title_left), top + m.bar),
    );
    Layout {
        outer: Rect::from_min_max(pos2(left, top), pos2(right, bottom)),
        elbow,
        bar,
        rail,
        cap,
        inner,
        title,
        slot,
    }
}


/// Chrome shapes for `l` (no text). `bg` must be the colour behind the group:
/// it carves the concave curve.
pub fn chrome_shapes(l: &Layout, m: &Metrics, color: Color32, bg: Color32) -> Vec<Shape> {
    let e = Elbow {
        corner: Corner::TL,
        thick: m.rail,
        thin: m.bar,
        r_out: m.r_out,
        r_in: m.r_in,
        color,
        bg,
    };
    let mut v = elbow_shapes(l.elbow, &e);
    if let Some(cap) = l.cap {
        let r = (m.bar / 2.0).clamp(0.0, 255.0) as u8;
        let cr = CornerRadius { nw: 0, sw: 0, ne: r, se: r };
        v.push(Shape::rect_filled(cap, cr, color));
    }
    v
}

/// Wrap `add` in a thin titled elbow. Contents get the remaining width.
pub fn group_box<R>(ui: &mut Ui, title: &str, color: Color32, add: impl FnOnce(&mut Ui) -> R) -> R {
    group_box_with(ui, &Metrics::default(), title, color, add)
}

pub fn group_box_with<R>(
    ui: &mut Ui,
    m: &Metrics,
    title: &str,
    color: Color32,
    add: impl FnOnce(&mut Ui) -> R,
) -> R {
    group_box_slot(ui, m, title, color, 0.0, add, |_| {})
}

/// The general form: `slot_w` > 0 opens a gap at the right end of the bar
/// and `slot` draws into it (right-aligned, vertically centred).
pub fn group_box_slot<R>(
    ui: &mut Ui,
    m: &Metrics,
    title: &str,
    color: Color32,
    slot_w: f32,
    add: impl FnOnce(&mut Ui) -> R,
    slot_ui: impl FnOnce(&mut Ui),
) -> R {
    group_box_full(ui, m, title, color, slot_w, None, add, slot_ui).0
}

/// What a click on the detach pill does, as a word (tooltip and screen readers).
pub fn detach_label(detached: bool) -> &'static str {
    if detached { "ATTACH" } else { "DETACH" }
}

/// Hover text for the detach toggle. The pill itself carries no text.
pub fn detach_tooltip(title: &str, detached: bool) -> String {
    if detached {
        format!("Attach: bring {} back here from its own window.", title.to_uppercase())
    } else {
        format!("Detach: open {} in its own window.", title.to_uppercase())
    }
}

/// Lowest fill/outline contrast the pill's two colours must reach.
pub const PILL_MIN_CONTRAST: f32 = 1.6;

/// The highlight of a pill colour: a lighter tint, or a darker shade when
/// the colour is already so light that a tint would not show.
pub fn pill_highlight(c: Color32) -> Color32 {
    let light = crate::tint(c, 0.5);
    if crate::contrast(c, light) >= PILL_MIN_CONTRAST {
        light
    } else {
        crate::tint(c, -0.45)
    }
}

/// (fill, outline) of the detach pill. Attached (raised): the elbow colour
/// with a highlight outline. Detached (pressed in): the two swap.
pub fn pill_colors(color: Color32, detached: bool) -> (Color32, Color32) {
    let hi = pill_highlight(color);
    if detached { (hi, color) } else { (color, hi) }
}

/// Width of the detach pill for a bar of metrics `m`: the wider of the two
/// labels plus padding, so the pill does not change size when it flips.
pub fn detach_pill_width(ui: &Ui, m: &Metrics) -> f32 {
    let font = FontId::proportional(m.title_size);
    let w = ["DETACH", "ATTACH"]
        .iter()
        .map(|t| ui.fonts_mut(|f| f.layout_no_wrap((*t).to_owned(), font.clone(), Color32::WHITE).size().x))
        .fold(0.0_f32, f32::max);
    (w + 2.0 * m.title_pad + m.bar / 2.0).ceil()
}

/// Thin border on the detach pill (same as a toggle border).
pub const PILL_BORDER: f32 = crate::TOGGLE_BORDER;

/// Paint and run the end pill of an elbow as the detach/attach toggle.
/// `rect` keeps the elbow-end shape (round right end). No text: attached it
/// is the elbow colour with a highlight outline; detached the colours swap
/// and a thin inner line shows it pressed in. The state is also in the
/// tooltip and the accessibility label.
///
/// Inside a native detached window, dragging the pill moves the window
/// (`ViewportCommand::StartDrag`, a real window-manager move, so the WM's
/// own snapping works); a click without movement still toggles.
pub fn detach_pill(ui: &mut Ui, rect: Rect, id: Id, color: Color32, title: &str, detached: bool) -> Response {
    let resp = ui.interact(rect, id, Sense::click_and_drag());
    if resp.drag_started() && in_native_detached_window(ui.ctx()) {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }
    let r = (rect.height() / 2.0).clamp(0.0, 255.0) as u8;
    let cr = CornerRadius { nw: 0, sw: 0, ne: r, se: r };
    let (fill, outline) = pill_colors(color, detached);
    let width = if resp.hovered() { PILL_BORDER + 0.5 } else { PILL_BORDER };
    let p = ui.painter();
    p.rect_filled(rect, cr, fill);
    p.rect_stroke(rect, cr, Stroke::new(width, outline), StrokeKind::Inside);
    if detached {
        let ri = r.saturating_sub(3);
        let icr = CornerRadius { nw: 0, sw: 0, ne: ri, se: ri };
        p.rect_stroke(rect.shrink(3.0), icr, Stroke::new(1.0, outline), StrokeKind::Inside);
    }
    if resp.has_focus() {
        p.rect_stroke(rect.expand(1.0), cr, Stroke::new(1.0, ink_on(fill)), StrokeKind::Outside);
    }
    let info = format!("{} {} ({})", detach_label(detached), title.to_uppercase(), if detached { "detached" } else { "attached" });
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, &info));
    let mut tip = detach_tooltip(title, detached);
    if in_native_detached_window(ui.ctx()) {
        tip.push_str(" Drag to move the window.");
    }
    resp.on_hover_text(tip)
}

/// [`group_box_slot`] whose end cap may be a detach pill: `pill` is the
/// pill's id and whether the group is detached now. Returns the contents'
/// value and the pill's response.
#[allow(clippy::too_many_arguments)]
pub fn group_box_full<R>(
    ui: &mut Ui,
    m: &Metrics,
    title: &str,
    color: Color32,
    slot_w: f32,
    pill: Option<(Id, bool)>,
    add: impl FnOnce(&mut Ui) -> R,
    slot_ui: impl FnOnce(&mut Ui),
) -> (R, Option<Response>) {
    let mut mm = *m;
    if pill.is_some() {
        mm.cap_w = detach_pill_width(ui, m);
        mm.cap_min_width = 0.0;
    }
    let m = &mm;
    let bg = ui.visuals().panel_fill;
    let slot = ui.painter().add(Shape::Noop);
    let avail = ui.available_rect_before_wrap();
    let (left, top, width) = (avail.left(), avail.top(), ui.available_width());
    let probe = layout_with_slot(left, top, width, 0.0, m, slot_w);
    let max = Rect::from_min_max(probe.inner.min, pos2(probe.inner.right(), f32::INFINITY));
    let mut child = ui.new_child(UiBuilder::new().max_rect(max).layout(*ui.layout()));
    let r = add(&mut child);
    let used = child.min_rect();
    let l = layout_with_slot(left, top, width, (used.bottom() - probe.inner.top()).max(0.0), m, slot_w);
    let resp = ui.allocate_rect(l.outer, Sense::hover());
    let label = title.to_owned();
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Label, true, &label));
    let mut chrome = l;
    if pill.is_some() {
        chrome.cap = None; // the pill paints its own cap
    }
    ui.painter().set(slot, Shape::Vec(chrome_shapes(&chrome, m, color, bg)));
    if l.title.width() > 0.0 {
        ui.painter().with_clip_rect(l.title).text(
            pos2(l.title.left(), l.title.center().y),
            Align2::LEFT_CENTER,
            title.to_uppercase(),
            FontId::proportional(m.title_size),
            ink_on(color),
        );
    }
    if let Some(rect) = l.slot {
        // A child, not `scope_builder`: a scope allocates its rect in the
        // parent and moves the cursor back up to the bar, so the next group
        // would be drawn on top of this one.
        let mut c = ui.new_child(
            UiBuilder::new()
                .max_rect(rect)
                .layout(egui::Layout::right_to_left(egui::Align::Center)),
        );
        slot_ui(&mut c);
    }
    let pill_resp = match (pill, l.cap) {
        (Some((id, detached)), Some(cap)) => Some(detach_pill(ui, cap, id, color, title, detached)),
        _ => None,
    };
    (r, pill_resp)
}

// ---------------------------------------------------------------- detach

/// Where a detachable elbow is: attached in its parent, or in its own window.
/// Kept in egui memory under the elbow's id (see [`DetachState::load`]); a
/// caller that persists it can read and write it with load/store.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DetachState {
    pub detached: bool,
    /// Last inner size of the detached window.
    pub size: Option<Vec2>,
    /// Last outer position of the detached window.
    pub pos: Option<egui::Pos2>,
    /// The detached window did not draw last frame (backend could not open
    /// it). Shown as plain text in the stub; cleared when it draws again.
    pub failed: bool,
}

impl DetachState {
    /// The memory id for a detachable elbow named by `salt`. Global, not
    /// per-`Ui`, so the same elbow is found from any window.
    pub fn id(salt: impl std::hash::Hash + std::fmt::Debug) -> Id {
        Id::NULL.with(("rustcars-detach", salt))
    }
    pub fn load(ctx: &egui::Context, id: Id) -> Self {
        ctx.data(|d| d.get_temp::<DetachState>(id)).unwrap_or_default()
    }
    pub fn store(self, ctx: &egui::Context, id: Id) {
        ctx.data_mut(|d| d.insert_temp(id, self));
    }
    /// Flip attached/detached.
    pub fn toggle(&mut self) {
        self.detached = !self.detached;
    }
}

fn native_window_flag_id() -> Id {
    Id::NULL.with("rustcars-detach-native-window")
}

/// True while drawing the contents of a detached window that is a real OS
/// window (not embedded in the parent). Pills there drag the window.
pub fn in_native_detached_window(ctx: &egui::Context) -> bool {
    let id = native_window_flag_id();
    ctx.data(|d| d.get_temp::<bool>(id)).unwrap_or(false)
}

/// Size of the resize grip in a detached window's bottom-right corner.
pub const RESIZE_GRIP: f32 = 14.0;

/// Keep a saved window position on screen: at least `keep` px of the window
/// stays inside a monitor of `monitor` size.
pub fn clamp_window_pos(pos: egui::Pos2, size: Vec2, monitor: Option<Vec2>) -> egui::Pos2 {
    let keep = vec2(120.0, 48.0);
    let Some(m) = monitor.filter(|m| m.x > keep.x && m.y > keep.y) else {
        return pos;
    };
    let w = size.x.max(keep.x);
    pos2(
        pos.x.clamp(keep.x - w, m.x - keep.x),
        pos.y.clamp(0.0, m.y - keep.y),
    )
}

/// The detached window's viewport id for elbow `id`.
pub fn detach_viewport_id(id: Id) -> ViewportId {
    ViewportId::from_hash_of(("rustcars-detach-vp", id))
}

/// Show `add` in its own window, wrapped in the same elbow with a pressed-in
/// pill. Clicking the pill or closing the window attaches it (sets
/// `state.detached = false`). Uses an immediate viewport, so `add` may borrow;
/// on a backend without multiple viewports egui embeds it as a window.
/// Call it every frame while `state.detached`; stop calling it to close.
///
/// `add` is `FnMut` on purpose: eframe runs an immediate viewport's callback
/// through `Context::run_ui`, which may run it twice in one frame (a second
/// pass after `request_discard`, up to `Options::max_passes`) and paints only
/// the last pass. So the contents are drawn in every pass. If the backend
/// could not open the window the callback does not run at all; then this
/// returns `None` and sets `state.failed`, and the next frame tries again.
pub fn detached_window<R>(
    ctx: &egui::Context,
    id: Id,
    m: &Metrics,
    title: &str,
    color: Color32,
    state: &mut DetachState,
    add: impl FnMut(&mut Ui) -> R,
) -> Option<R> {
    detached_window_in(ctx, id, m, title, color, state, add, |vp, builder, cb| {
        ctx.show_viewport_immediate(vp, builder, |ui, class| cb(ui, class == egui::ViewportClass::Immediate));
    })
}

/// [`detached_window`] with the viewport runner passed in. `run` must call
/// `cb` with the window's `Ui` (and whether it is a native OS window) zero or
/// more times; tests use it to act like a backend that runs the callback
/// twice or not at all.
#[doc(hidden)]
#[allow(clippy::too_many_arguments)]
pub fn detached_window_in<R>(
    ctx: &egui::Context,
    id: Id,
    m: &Metrics,
    title: &str,
    color: Color32,
    state: &mut DetachState,
    mut add: impl FnMut(&mut Ui) -> R,
    run: impl FnOnce(ViewportId, ViewportBuilder, &mut dyn FnMut(&mut Ui, bool)),
) -> Option<R> {
    let _ = ctx;
    // Borderless: the elbow is the window's frame and its pill the handle.
    // Resizable with only a small minimum, so the window manager can snap it.
    let mut builder = ViewportBuilder::default()
        .with_title(title.to_uppercase())
        .with_decorations(false)
        .with_resizable(true)
        .with_inner_size(state.size.unwrap_or(vec2(720.0, 480.0)))
        .with_min_inner_size([240.0, 120.0]);
    if let Some(pos) = state.pos {
        builder = builder.with_position(pos);
    }
    let pill_id = id.with("pill-window");
    let mut out: Option<R> = None;
    let mut ran = false;
    let mut attach = false;
    let mut size = None;
    let mut pos = None;
    let flag = native_window_flag_id();
    let mut cb = |ui: &mut Ui, native: bool| {
        ran = true;
        let was = ui.ctx().data(|d| d.get_temp::<bool>(flag)).unwrap_or(false);
        ui.ctx().data_mut(|d| d.insert_temp(flag, native));
        let closing = ui.input(|i| i.viewport().close_requested());
        let info = ui.input(|i| i.viewport().clone());
        if closing {
            attach = true;
        }
        egui::Frame::central_panel(ui.style()).show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt(("rustcars-detach-scroll", id))
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let (r, pill) = group_box_full(ui, m, title, color, 0.0, Some((pill_id, true)), &mut add, |_| {});
                    // Every pass draws the contents; keep the latest value.
                    out = Some(r);
                    if pill.is_some_and(|p| p.clicked()) {
                        attach = true;
                    }
                });
        });
        if native {
            // No OS border: outline the window and give it a resize grip.
            let full = ui.ctx().content_rect();
            let stroke = Stroke::new(PILL_BORDER, color);
            ui.painter().rect_stroke(full, 0.0, stroke, StrokeKind::Inside);
            let grip = Rect::from_min_max(full.max - vec2(RESIZE_GRIP, RESIZE_GRIP), full.max);
            let g = ui.interact(grip, id.with("resize-grip"), Sense::drag());
            let g = g.on_hover_cursor(egui::CursorIcon::ResizeSouthEast).on_hover_text("Drag to resize");
            for k in [4.0, 8.0, 12.0] {
                ui.painter().line_segment(
                    [pos2(full.max.x - k, full.max.y - 2.0), pos2(full.max.x - 2.0, full.max.y - k)],
                    Stroke::new(1.0, pill_highlight(color)),
                );
            }
            if g.drag_started() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::BeginResize(egui::ResizeDirection::SouthEast));
            }
        }
        if let Some(s) = info.inner_rect.map(|r| r.size()).filter(|s| s.x >= 64.0 && s.y >= 64.0) {
            size = Some(s);
        }
        if let Some(p) = info.outer_rect.map(|r| r.min) {
            let sz = info.outer_rect.map(|r| r.size()).unwrap_or(vec2(720.0, 480.0));
            pos = Some(clamp_window_pos(p, sz, info.monitor_size));
        }
        ui.ctx().data_mut(|d| d.insert_temp(flag, was));
    };
    run(detach_viewport_id(id), builder, &mut cb);
    if size.is_some() {
        state.size = size;
    }
    if pos.is_some() {
        state.pos = pos;
    }
    state.failed = !ran;
    if attach {
        state.detached = false;
        state.failed = false;
    }
    out
}

/// Stub text when the detached window could not be drawn.
pub fn detach_failed_text(title: &str) -> String {
    format!(
        "{}: detached window failed: the window system did not draw it; click the pill to bring it back.",
        title.to_uppercase()
    )
}

/// Text left in the parent while an elbow is in its own window.
pub fn detached_stub_text(title: &str) -> String {
    format!("{}: detached, in its own window. Click the end pill to bring it back.", title.to_uppercase())
}

/// A titled LCARS group box (thin elbow). `.detachable(salt)` turns its end
/// pill into a detach/attach toggle: detached, the elbow and everything in
/// it move to their own window and a plain-text stub stays here.
pub struct GroupBox<'a> {
    title: &'a str,
    color: Color32,
    metrics: Metrics,
    detach: Option<Id>,
}

impl<'a> GroupBox<'a> {
    pub fn new(title: &'a str, color: Color32) -> Self {
        Self { title, color, metrics: Metrics::default(), detach: None }
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
    /// Make the end pill a detach toggle. `salt` names the elbow app-wide.
    pub fn detachable(mut self, salt: impl std::hash::Hash + std::fmt::Debug) -> Self {
        self.detach = Some(DetachState::id(salt));
        self
    }
    /// Draw it. Returns the contents' value, or `None` in a frame where the
    /// detached window did not draw (the stub then says so in plain text;
    /// the next frame tries again). Attached, the contents run once here.
    /// Detached, they run in the window, once per egui pass of that window
    /// (eframe may run two passes and paints the last; see
    /// [`detached_window`]), never in the parent. Hence `FnMut`.
    pub fn show<R>(self, ui: &mut Ui, add: impl FnMut(&mut Ui) -> R) -> Option<R> {
        let ctx = ui.ctx().clone();
        self.show_in(ui, add, |vp, builder, cb| {
            ctx.show_viewport_immediate(vp, builder, |ui, class| cb(ui, class == egui::ViewportClass::Immediate));
        })
    }

    /// [`Self::show`] with the viewport runner passed in (tests).
    #[doc(hidden)]
    pub fn show_in<R>(
        self,
        ui: &mut Ui,
        mut add: impl FnMut(&mut Ui) -> R,
        run: impl FnOnce(ViewportId, ViewportBuilder, &mut dyn FnMut(&mut Ui, bool)),
    ) -> Option<R> {
        let Some(id) = self.detach else {
            return Some(group_box_with(ui, &self.metrics, self.title, self.color, add));
        };
        let ctx = ui.ctx().clone();
        let mut st = DetachState::load(&ctx, id);
        let pill_id = id.with("pill");
        if !st.detached {
            let (r, pill) = group_box_full(ui, &self.metrics, self.title, self.color, 0.0, Some((pill_id, false)), &mut add, |_| {});
            if pill.is_some_and(|p| p.clicked()) {
                st.detached = true;
                st.failed = false;
                st.store(&ctx, id);
            }
            return Some(r);
        }
        let stub = if st.failed { detach_failed_text(self.title) } else { detached_stub_text(self.title) };
        let ((), pill) = group_box_full(ui, &self.metrics, self.title, self.color, 0.0, Some((pill_id, true)), |ui| {
            ui.label(stub);
        }, |_| {});
        let out = detached_window_in(&ctx, id, &self.metrics, self.title, self.color, &mut st, add, run);
        if pill.is_some_and(|p| p.clicked()) {
            st.detached = false;
            st.failed = false;
        }
        st.store(&ctx, id);
        out
    }
}
