//! The detach pill through egui's real immediate-viewport path: one shared
//! `Context`, `set_embed_viewports(false)` and an immediate viewport renderer
//! that runs the window's pass nested in the parent's pass with the window's
//! own input, as eframe's glow backend does. The parent may run two passes.
use egui::{Event, PointerButton, Pos2, RawInput, Rect, Shape, ViewportId, pos2, vec2};
use rustcars::group::*;
use rustcars::{LcarsTheme, apply_theme, install_lcars_font, palette};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
struct Win {
    events: Vec<Event>,
    time: f64,
    /// Where the window manager has the window (outer = inner, borderless).
    at: Pos2,
    size: Option<egui::Vec2>,
    builders: Vec<egui::ViewportBuilder>,
    shapes: Vec<egui::epaint::ClippedShape>,
    commands: Vec<egui::ViewportCommand>,
    runs: u32,
    maximized: bool,
}

fn vps(id: ViewportId, size: egui::Vec2) -> egui::ViewportIdMap<egui::ViewportInfo> {
    vps_at(id, Pos2::ZERO, size)
}

fn vps_at(id: ViewportId, at: Pos2, size: egui::Vec2) -> egui::ViewportIdMap<egui::ViewportInfo> {
    let r = Rect::from_min_size(at, size);
    let info = egui::ViewportInfo { inner_rect: Some(r), outer_rect: Some(r), ..Default::default() };
    std::iter::once((id, info)).collect()
}

fn setup() -> (egui::Context, Rc<RefCell<Win>>) {
    let ctx = egui::Context::default();
    install_lcars_font(&ctx);
    apply_theme(&ctx, &LcarsTheme::default());
    ctx.set_embed_viewports(false);
    let win = Rc::new(RefCell::new(Win::default()));
    let w = Rc::clone(&win);
    egui::Context::set_immediate_viewport_renderer(move |ctx, vp| {
        w.borrow_mut().builders.push(vp.builder.clone());
        let mut cb = vp.viewport_ui_cb;
        let events = std::mem::take(&mut w.borrow_mut().events);
        let time = w.borrow().time;
        let at = w.borrow().at;
        let size = w.borrow().size.unwrap_or(vec2(720.0, 480.0));
        let mut viewports = vps_at(vp.ids.this, at, size);
        let max = w.borrow().maximized;
        for i in viewports.values_mut() {
            i.maximized = Some(max);
            i.focused = Some(true);
            i.monitor_size = Some(vec2(1920.0, 1080.0));
        }
        let input = RawInput {
            time: Some(time),
            viewport_id: vp.ids.this,
            viewports,
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
            events,
            ..Default::default()
        };
        let mut out = ctx.run_ui(input, |ui| cb(ui));
        out.textures_delta.clear();
        let mut w = w.borrow_mut();
        w.runs += 1;
        w.shapes = out.shapes;
        w.commands = out.viewport_output.values().flat_map(|v| v.commands.clone()).collect();
    });
    (ctx, win)
}

const TITLE: &str = "REC";

/// One parent frame like Commander: the main pill in the page's top cap,
/// then the detached window while detached. `two_passes` mimics a parent
/// that requests a discard (eframe then runs the parent, and with it the
/// immediate window, twice).
fn frame(ctx: &egui::Context, main_events: Vec<Event>, two_passes: bool) -> Rect {
    let id = DetachState::id(("page", TITLE));
    let mut main_pill = Rect::NOTHING;
    let mut pass = 0;
    let mut out = ctx.run_ui(
        RawInput {
            viewport_id: ViewportId::ROOT,
            viewports: vps(ViewportId::ROOT, vec2(1200.0, 800.0)),
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(1200.0, 800.0))),
            events: main_events,
            ..Default::default()
        },
        |ui| {
            if two_passes && pass == 0 {
                ui.ctx().request_discard("test: second pass");
            }
            pass += 1;
            let mut st = DetachState::load(ui.ctx(), id);
            let cap = Rect::from_min_size(pos2(1100.0, 0.0), vec2(80.0, 24.0));
            main_pill = cap;
            if detach_pill(ui, cap, id.with("page-pill"), palette::TAN, TITLE, st.detached).clicked() {
                st.toggle();
                st.store(ui.ctx(), id);
            }
            let mut st = DetachState::load(ui.ctx(), id);
            if st.detached {
                let m = Metrics::default();
                detached_window(ui.ctx(), id, &m, TITLE, palette::TAN, &mut st, |ui| {
                    if ui.button("CONTENT").clicked() {
                        CONTENT_CLICKS.with(|c| *c.borrow_mut() += 1);
                    }
                });
                st.store(ui.ctx(), id);
            }
        },
    );
    out.textures_delta.clear();
    let root_cmds: Vec<_> = out.viewport_output.iter().map(|(id, v)| (*id, v.commands.clone())).collect();
    ROOT_CMDS.with(|c| c.borrow_mut().extend(root_cmds));
    main_pill
}

thread_local! {
    static CONTENT_CLICKS: RefCell<u32> = const { RefCell::new(0) };
}

thread_local! {
    static ROOT_CMDS: RefCell<Vec<(ViewportId, Vec<egui::ViewportCommand>)>> = RefCell::new(vec![]);
}

fn detached(ctx: &egui::Context) -> bool {
    DetachState::load(ctx, DetachState::id(("page", TITLE))).detached
}

fn window_pill(win: &Rc<RefCell<Win>>) -> Rect {
    win.borrow()
        .shapes
        .iter()
        .filter_map(|c| match &c.shape {
            Shape::Rect(r) if r.fill == pill_colors(palette::TAN, true).0 && r.rect.width() < 200.0 && r.rect.height() < 40.0 => Some(r.rect),
            _ => None,
        })
        .next()
        .expect("window pill painted")
}

fn btn(p: Pos2, pressed: bool) -> Event {
    Event::PointerButton { pos: p, button: PointerButton::Primary, pressed, modifiers: Default::default() }
}

fn detach_from_main(ctx: &egui::Context, win: &Rc<RefCell<Win>>, two: bool) {
    let cap = frame(ctx, vec![], two);
    let p = cap.center();
    frame(ctx, vec![Event::PointerMoved(p)], two);
    frame(ctx, vec![btn(p, true)], two);
    frame(ctx, vec![btn(p, false)], two);
    frame(ctx, vec![], two);
    assert!(detached(ctx), "main pill detaches");
    assert!(win.borrow().runs > 0, "window drawn");
}

fn dock_by_window_click(two: bool) {
    let (ctx, win) = setup();
    detach_from_main(&ctx, &win, two);
    frame(&ctx, vec![], two);
    let p = window_pill(&win).center();
    for ev in [vec![Event::PointerMoved(p)], vec![btn(p, true)], vec![btn(p, false)], vec![]] {
        win.borrow_mut().events = ev;
        frame(&ctx, vec![], two);
        assert!(!win.borrow().commands.contains(&egui::ViewportCommand::StartDrag), "a click is not a drag");
    }
    assert!(!detached(&ctx), "clicking the window pill docks (two passes: {two})");
}

#[test]
fn window_pill_click_docks_one_pass() {
    dock_by_window_click(false);
}

#[test]
fn window_pill_click_docks_two_passes() {
    dock_by_window_click(true);
}

#[test]
fn window_pill_drag_moves_and_does_not_dock() {
    for two in [false, true] {
        let (ctx, win) = setup();
        detach_from_main(&ctx, &win, two);
        frame(&ctx, vec![], two);
        let p = window_pill(&win).center();
        let mut cmds = vec![];
        for ev in [vec![Event::PointerMoved(p)], vec![btn(p, true)], vec![Event::PointerMoved(p + vec2(30.0, 10.0))], vec![Event::PointerMoved(p + vec2(60.0, 20.0))], vec![btn(p + vec2(60.0, 20.0), false)], vec![]] {
            win.borrow_mut().events = ev;
            frame(&ctx, vec![], two);
            cmds.extend(win.borrow().commands.clone());
        }
        let root: Vec<_> = ROOT_CMDS.with(|c| c.borrow().iter().flat_map(|(_, v)| v.clone()).collect());
        assert!(cmds.contains(&egui::ViewportCommand::StartDrag) || root.contains(&egui::ViewportCommand::StartDrag), "drag moves the window");
        assert!(detached(&ctx), "a drag does not dock");
    }
}

#[test]
fn main_pill_click_docks_too() {
    let (ctx, win) = setup();
    detach_from_main(&ctx, &win, true);
    let cap = frame(&ctx, vec![], true);
    let p = cap.center();
    for ev in [vec![Event::PointerMoved(p)], vec![btn(p, true)], vec![btn(p, false)], vec![]] {
        frame(&ctx, ev, true);
    }
    assert!(!detached(&ctx), "main pill docks");
}

/// The window only runs when the parent repaints, so a quick click arrives
/// as press + release in one window pass (and the move with it).
#[test]
fn window_pill_quick_click_in_one_pass_docks() {
    for (two, hover_first) in [(false, false), (true, false), (false, true), (true, true)] {
        let (ctx, win) = setup();
        detach_from_main(&ctx, &win, two);
        frame(&ctx, vec![], two);
        let p = window_pill(&win).center();
        if hover_first {
            win.borrow_mut().events = vec![Event::PointerMoved(p)];
            frame(&ctx, vec![], two);
            win.borrow_mut().events = vec![btn(p, true), btn(p, false)];
        } else {
            win.borrow_mut().events = vec![Event::PointerMoved(p), btn(p, true), btn(p, false)];
        }
        frame(&ctx, vec![], two);
        frame(&ctx, vec![], two);
        assert!(!detached(&ctx), "one-pass click docks (two passes {two}, hover first {hover_first})");
    }
}

/// Real time passes: the pointer rests on the pill long enough for its
/// tooltip, then clicks. The tooltip must not take the click.
#[test]
fn window_pill_click_after_tooltip_docks() {
    for two in [false, true] {
        let (ctx, win) = setup();
        detach_from_main(&ctx, &win, two);
        frame(&ctx, vec![], two);
        let p = window_pill(&win).center();
        let mut t = 1.0;
        win.borrow_mut().events = vec![Event::PointerMoved(p)];
        for _ in 0..40 {
            t += 0.05;
            win.borrow_mut().time = t;
            frame(&ctx, vec![], two);
        }
        for ev in [vec![btn(p, true)], vec![btn(p, false)], vec![], vec![]] {
            t += 0.05;
            win.borrow_mut().time = t;
            win.borrow_mut().events = ev;
            frame(&ctx, vec![], two);
        }
        assert!(!detached(&ctx), "click after the tooltip shows docks (two passes {two})");
    }
}

/// The pointer enters the window and presses in the same window pass, the
/// release comes a pass later (the common real-world click).
#[test]
fn window_pill_enter_and_press_together_then_release_docks() {
    for two in [false, true] {
        let (ctx, win) = setup();
        detach_from_main(&ctx, &win, two);
        frame(&ctx, vec![], two);
        let p = window_pill(&win).center();
        win.borrow_mut().events = vec![Event::PointerGone];
        frame(&ctx, vec![], two);
        let mut cmds = vec![];
        for (dt, ev) in [(0.05, vec![Event::PointerMoved(p), btn(p, true)]), (0.1, vec![btn(p, false)]), (0.05, vec![])] {
            let t = win.borrow().time + dt;
            win.borrow_mut().time = t;
            win.borrow_mut().events = ev;
            frame(&ctx, vec![], two);
            cmds.extend(ROOT_CMDS.with(|c| c.borrow_mut().drain(..).flat_map(|(_, v)| v).collect::<Vec<_>>()));
        }
        assert!(!cmds.contains(&egui::ViewportCommand::StartDrag), "a click is not a window move (two {two}): {cmds:?}");
        assert!(!detached(&ctx), "enter+press then release docks (two passes {two})");
    }
}

fn place_cmds(cmds: &[egui::ViewportCommand]) -> Vec<egui::ViewportCommand> {
    cmds.iter()
        .filter(|c| matches!(c, egui::ViewportCommand::OuterPosition(_) | egui::ViewportCommand::InnerSize(_)))
        .cloned()
        .collect()
}

/// While the window manager moves or resizes the window (after StartDrag /
/// BeginResize), egui must not send the window back to where it was.
#[test]
fn window_manager_move_and_resize_are_not_fought() {
    for two in [false, true] {
        let (ctx, win) = setup();
        detach_from_main(&ctx, &win, two);
        win.borrow_mut().at = pos2(300.0, 200.0);
        for _ in 0..3 {
            frame(&ctx, vec![], two);
        }
        ROOT_CMDS.with(|c| c.borrow_mut().clear());
        for i in 1..=10 {
            win.borrow_mut().at = pos2(300.0 + 25.0 * i as f32, 200.0 + 10.0 * i as f32);
            win.borrow_mut().size = Some(vec2(720.0 + 10.0 * i as f32, 480.0 + 5.0 * i as f32));
            frame(&ctx, vec![], two);
        }
        let cmds: Vec<_> = ROOT_CMDS.with(|c| c.borrow_mut().drain(..).flat_map(|(_, v)| v).collect());
        assert!(place_cmds(&cmds).is_empty(), "egui fights the window manager (two {two}): {:?}", place_cmds(&cmds));
        let st = DetachState::load(&ctx, DetachState::id(("page", TITLE)));
        assert_eq!(st.pos, Some(pos2(550.0, 300.0)), "the last place is remembered");
    }
}

/// Root cause of the fight: the saved place went into the builder every
/// frame, and eframe patches a live window to any changed builder place.
/// Now only the opening frame carries it; a reopen carries the last place.
#[test]
fn builder_places_the_window_only_when_it_opens() {
    for two in [false, true] {
        let (ctx, win) = setup();
        let id = DetachState::id(("page", TITLE));
        DetachState { pos: Some(pos2(40.0, 50.0)), size: Some(vec2(800.0, 500.0)), ..Default::default() }.store(&ctx, id);
        detach_from_main(&ctx, &win, two);
        let b = win.borrow().builders.clone();
        assert_eq!(b[0].position, Some(pos2(40.0, 50.0)), "opens at the saved place");
        assert_eq!(b[0].inner_size, Some(vec2(800.0, 500.0)));
        assert!(b[1..].iter().all(|b| b.position.is_none() && b.inner_size.is_none()), "later frames leave the window to the WM");
        // The WM moves it, the user docks and detaches again: it reopens there.
        win.borrow_mut().at = pos2(400.0, 300.0);
        frame(&ctx, vec![], two);
        let st = DetachState::load(&ctx, id);
        let mut st2 = st;
        st2.toggle();
        st2.store(&ctx, id);
        frame(&ctx, vec![], two);
        assert!(!detached(&ctx));
        let mut st3 = DetachState::load(&ctx, id);
        st3.toggle();
        st3.store(&ctx, id);
        win.borrow_mut().builders.clear();
        frame(&ctx, vec![], two);
        assert_eq!(win.borrow().builders[0].position, Some(pos2(400.0, 300.0)), "reopens where it was");
    }
}


fn content_rect(win: &Rc<RefCell<Win>>) -> Rect {
    win.borrow()
        .shapes
        .iter()
        .filter_map(|c| match &c.shape {
            Shape::Text(t) if t.galley.text() == "CONTENT" => Some(t.visual_bounding_rect()),
            _ => None,
        })
        .next()
        .expect("content painted")
}

/// Maximized, half and quarter snapped, then restored: a click on the
/// window's content still reaches it every time, through the real nested
/// immediate-viewport path with the window's own input.
#[test]
fn content_takes_clicks_maximized_snapped_and_restored() {
    let (ctx, win) = setup();
    detach_from_main(&ctx, &win, false);
    assert!(detached(&ctx));
    for (at, size, max) in [
        (pos2(0.0, 0.0), vec2(1920.0, 1050.0), true),
        (pos2(0.0, 30.0), vec2(960.0, 1050.0), false),
        (pos2(960.0, 30.0), vec2(960.0, 525.0), false),
        (pos2(300.0, 200.0), vec2(720.0, 480.0), false),
    ] {
        {
            let mut w = win.borrow_mut();
            w.at = at;
            w.size = Some(size);
            w.maximized = max;
        }
        frame(&ctx, vec![], false);
        frame(&ctx, vec![], false);
        let r = content_rect(&win);
        assert!(Rect::from_min_size(Pos2::ZERO, size).contains_rect(r), "content inside the window at {size:?}: {r:?}");
        let before = CONTENT_CLICKS.with(|c| *c.borrow());
        let p = r.center();
        for ev in [vec![Event::PointerMoved(p)], vec![btn(p, true)], vec![btn(p, false)], vec![]] {
            win.borrow_mut().time += 0.05;
            win.borrow_mut().events = ev;
            frame(&ctx, vec![], false);
        }
        let after = CONTENT_CLICKS.with(|c| *c.borrow());
        assert_eq!(after, before + 1, "click reached the content at {size:?} maximized={max}");
        assert!(detached(&ctx), "a content click does not dock");
    }
}


/// The window manager maximizes the detached window (super+Up): the
/// window's own pass sends Maximized(false) at once; snaps are left alone.
#[test]
fn a_maximized_detached_window_is_restored_and_snaps_are_not() {
    let (ctx, win) = setup();
    detach_from_main(&ctx, &win, false);
    {
        let mut w = win.borrow_mut();
        w.at = pos2(0.0, 0.0);
        w.size = Some(vec2(1920.0, 1050.0));
        w.maximized = true;
    }
    frame(&ctx, vec![], false);
    assert!(win.borrow().commands.contains(&egui::ViewportCommand::Maximized(false)), "{:?}", win.borrow().commands);
    win.borrow_mut().maximized = false;
    win.borrow_mut().size = Some(vec2(960.0, 1050.0));
    frame(&ctx, vec![], false);
    assert!(!win.borrow().commands.iter().any(|c| matches!(c, egui::ViewportCommand::Maximized(_))), "half snap untouched");
    let b = win.borrow().builders.last().cloned().unwrap();
    assert_eq!(b.maximize_button, Some(false));
}
