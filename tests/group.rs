//! Thin group elbows and the detach pill, with the real Antonio font and theme.
use egui::{Event, PointerButton, Pos2, Rect, RawInput, Shape, pos2, vec2};
use rustcars::group::*;
use rustcars::{LcarsTheme, apply_theme, install_lcars_font, palette};
use std::cell::Cell;

fn ctx() -> egui::Context {
    let ctx = egui::Context::default();
    install_lcars_font(&ctx);
    apply_theme(&ctx, &LcarsTheme::default());
    ctx
}

fn run(ctx: &egui::Context, i: RawInput, f: impl FnMut(&mut egui::Ui)) -> egui::FullOutput {
    let mut o = ctx.run_ui(i, f);
    o.textures_delta.clear();
    o
}

fn input(events: Vec<Event>) -> RawInput {
    RawInput {
        screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1000.0, 1400.0))),
        events,
        ..Default::default()
    }
}

/// Every text shape: (text, rect).
fn texts(out: &egui::FullOutput) -> Vec<(String, Rect)> {
    out.shapes
        .iter()
        .filter_map(|c| match &c.shape {
            Shape::Text(t) => Some((t.galley.text().to_owned(), Rect::from_min_size(t.pos, t.galley.size()))),
            _ => None,
        })
        .collect()
}

fn click_at(p: Pos2) -> Vec<Vec<Event>> {
    let b = |pressed| Event::PointerButton { pos: p, button: PointerButton::Primary, pressed, modifiers: Default::default() };
    vec![vec![Event::PointerMoved(p)], vec![b(true)], vec![b(false)], vec![]]
}

#[test]
fn labels_and_toggle() {
    assert_eq!(detach_label(false), "DETACH");
    assert_eq!(detach_label(true), "ATTACH");
    let mut s = DetachState::default();
    assert!(!s.detached);
    s.toggle();
    assert!(s.detached);
    s.toggle();
    assert!(!s.detached);
    assert!(detached_stub_text("Audio patch").contains("AUDIO PATCH: detached"));
    assert_ne!(DetachState::id("a"), DetachState::id("b"));
}

#[test]
fn pill_fits_label_in_real_font_and_keeps_bar_height() {
    let ctx = ctx();
    let mut out = None;
    for _ in 0..2 {
        out = Some(run(&ctx, input(vec![]), |ui| {
            GroupBox::new("RF patch", palette::TAN).detachable("rf").show(ui, |ui| ui.label("x"));
        }));
    }
    let t = texts(out.as_ref().unwrap());
    let (_, r) = t.iter().find(|(s, _)| s == "DETACH").expect("DETACH pill text");
    let m = Metrics::default();
    // pill is at the right end of the bar, bar height, text inside it
    assert!(r.top() >= -0.5 && r.bottom() <= m.bar + 0.5, "text {r:?} outside the 16 px bar");
    let _ = run(&ctx, input(vec![]), |ui| {
        let pw = detach_pill_width(ui, &m);
        let right = ui.available_rect_before_wrap().right();
        ui.data_mut(|d| d.insert_temp(egui::Id::NULL.with("pw"), (pw, right)));
    });
    let (pw, right): (f32, f32) = ctx.data(|d| d.get_temp(egui::Id::NULL.with("pw"))).unwrap();
    assert!(r.right() <= right + 0.5 && r.left() >= right - pw - 0.5, "text {r:?} not in pill (w {pw})");
    assert!(r.width() + 4.0 <= pw);
}

/// Run frames clicking at `p`; count how many times `content` ran per frame.
#[test]
fn click_pill_detaches_and_attaches_content_once_per_frame() {
    let ctx = ctx();
    let runs = Cell::new(0);
    let frame = |ctx: &egui::Context, ev: Vec<Event>| {
        runs.set(0);
        let out = run(ctx, input(ev), |ui| {
            GroupBox::new("RF patch", palette::TAN).detachable("rf").show(ui, |ui| {
                runs.set(runs.get() + 1);
                ui.label("CONTENT");
            });
        });
        assert_eq!(runs.get(), 1, "content must run exactly once");
        out
    };
    frame(&ctx, vec![]);
    let mut out = frame(&ctx, vec![]);
    let pill = texts(&out).into_iter().find(|(s, _)| s == "DETACH").unwrap().1;
    for ev in click_at(pill.center()) {
        frame(&ctx, ev);
    }
    // A new window sits out its first (sizing) frame.
    frame(&ctx, vec![]);
    out = frame(&ctx, vec![]);
    let id = DetachState::id("rf");
    assert!(DetachState::load(&ctx, id).detached, "click on pill detaches");
    let t = texts(&out);
    assert!(t.iter().any(|(s, _)| s.contains("RF PATCH: detached")), "stub text shown");
    let attach: Vec<Rect> = t.iter().filter(|(s, _)| s == "ATTACH").map(|x| x.1).collect();
    assert_eq!(attach.len(), 2, "stub pill and window pill both read ATTACH");
    let content = t.iter().filter(|(s, _)| s == "CONTENT").count();
    assert_eq!(content, 1, "content drawn once (in the window)");
    // Click the window's pill (the one that is not where the stub's is).
    let win = attach.iter().find(|r| (r.center() - pill.center()).length() > 2.0).copied().unwrap();
    for ev in click_at(win.center()) {
        frame(&ctx, ev);
    }
    assert!(!DetachState::load(&ctx, id).detached, "window pill attaches");
    out = frame(&ctx, vec![]);
    let t = texts(&out);
    assert!(t.iter().any(|(s, _)| s == "DETACH"));
    assert!(!t.iter().any(|(s, _)| s.contains("detached")));
}

#[test]
fn groups_stack_without_overlap_in_both_modes() {
    let ctx = ctx();
    for detached in [false, true] {
        for i in 0..6 {
            DetachState { detached: detached && i % 2 == 0, ..Default::default() }.store(&ctx, DetachState::id(("g", i)));
        }
        let mut rects = Vec::new();
        for _ in 0..2 {
            rects.clear();
            let _ = run(&ctx, input(vec![]), |ui| {
                for i in 0..6 {
                    let before = ui.cursor().top();
                    let g = GroupBox::new("G", palette::OLIVE);
                    let g = if i % 3 == 2 { g } else { g.detachable(("g", i)) };
                    g.show(ui, |ui| {
                        for _ in 0..4 {
                            ui.label("row");
                        }
                    });
                    rects.push((before, ui.min_rect().bottom()));
                }
            });
        }
        for w in rects.windows(2) {
            assert!(w[1].0 >= w[0].1 - 0.5, "overlap (detached={detached}): {rects:?}");
            assert!(w[0].1 - w[0].0 > 30.0, "collapsed (detached={detached}): {rects:?}");
        }
    }
}

/// A page elbow holding detachable sub-elbows, Commander's Config layout.
#[test]
fn nested_detach_page_and_sub_panels() {
    let ctx = ctx();
    let (page, audio, rf) = (Cell::new(0), Cell::new(0), Cell::new(0));
    let frame = |ctx: &egui::Context| {
        page.set(0);
        audio.set(0);
        rf.set(0);
        let out = run(ctx, input(vec![]), |ui| {
            GroupBox::new("Config", palette::TAN).detachable("config").show(ui, |ui| {
                page.set(page.get() + 1);
                GroupBox::new("Audio patch", palette::SLATE).detachable("audio").show(ui, |ui| {
                    audio.set(audio.get() + 1);
                    ui.label("AUDIO-BODY");
                });
                GroupBox::new("RF patch", palette::OLIVE).detachable("rf").show(ui, |ui| {
                    rf.set(rf.get() + 1);
                    ui.label("RF-BODY");
                });
            });
        });
        assert_eq!((page.get(), audio.get(), rf.get()), (1, 1, 1), "each body exactly once");
        let t = texts(&out);
        let n = |s: &str| t.iter().filter(|(x, _)| x == s).count();
        let stub = |s: &str| t.iter().any(|(x, _)| x.starts_with(s) && x.contains("detached"));
        (n("AUDIO-BODY"), n("RF-BODY"), stub("CONFIG:"), stub("AUDIO PATCH:"), stub("RF PATCH:"))
    };
    let set = |salt: &str, d: bool| DetachState { detached: d, ..Default::default() }.store(&ctx, DetachState::id(salt));
    frame(&ctx);
    assert_eq!(frame(&ctx), (1, 1, false, false, false));
    // Sub-panel out of an attached page: stub in the page.
    set("audio", true);
    frame(&ctx);
    assert_eq!(frame(&ctx), (1, 1, false, true, false));
    // Whole page out: RF goes with it, Audio stays in its own window.
    set("config", true);
    frame(&ctx);
    assert_eq!(frame(&ctx), (1, 1, true, true, false));
    // Sub-panel back while the page is out: it lands in the page window.
    set("audio", false);
    frame(&ctx);
    assert_eq!(frame(&ctx), (1, 1, true, false, false));
    // Page back: all inline again.
    set("config", false);
    frame(&ctx);
    assert_eq!(frame(&ctx), (1, 1, false, false, false));
}
