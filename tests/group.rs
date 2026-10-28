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

/// Pill-shaped filled rects (square left, round right): (rect, fill).
fn pills(out: &egui::FullOutput) -> Vec<(Rect, egui::Color32)> {
    out.shapes
        .iter()
        .filter_map(|c| match &c.shape {
            Shape::Rect(r) if r.corner_radius.nw == 0 && r.corner_radius.ne > 0 && r.fill != egui::Color32::TRANSPARENT => {
                Some((r.rect, r.fill))
            }
            _ => None,
        })
        .collect()
}

/// Outline colour of the stroke painted on `rect`.
fn outline_at(out: &egui::FullOutput, rect: Rect) -> Option<egui::Color32> {
    out.shapes.iter().find_map(|c| match &c.shape {
        Shape::Rect(r) if r.rect == rect && r.fill == egui::Color32::TRANSPARENT && r.stroke.width > 0.0 => Some(r.stroke.color),
        _ => None,
    })
}

#[test]
fn pill_colours_swap_and_stay_distinct_on_every_hue() {
    let hues = [
        palette::TAN, palette::SLATE, palette::OLIVE, palette::BRICK, palette::TEAL, palette::PLUM,
        palette::TEXT, palette::TEXT_MUTED, egui::Color32::from_rgb(0xD9, 0x42, 0x3A),
        egui::Color32::from_rgb(0xFF, 0x99, 0x00), egui::Color32::WHITE, egui::Color32::from_gray(0x60),
    ];
    for c in hues {
        let (f0, o0) = pill_colors(c, false);
        let (f1, o1) = pill_colors(c, true);
        assert_eq!(f0, c);
        assert_eq!((f1, o1), (o0, f0), "colours swap for {c:?}");
        assert!(rustcars::contrast(f0, o0) >= PILL_MIN_CONTRAST, "{c:?} fill/outline too close");
    }
}

/// Run detachable RF patch frames; content must run once each frame.
fn rf_frame(ctx: &egui::Context, ev: Vec<Event>, runs: &Cell<u32>) -> egui::FullOutput {
    runs.set(0);
    let out = run(ctx, input(ev), |ui| {
        GroupBox::new("RF patch", palette::TAN).detachable("rf").show(ui, |ui| {
            runs.set(runs.get() + 1);
            ui.label("CONTENT");
        });
    });
    assert_eq!(runs.get(), 1, "content must run exactly once");
    out
}

#[test]
fn pill_has_no_text_fixed_size_and_click_toggles() {
    let ctx = ctx();
    let runs = Cell::new(0);
    rf_frame(&ctx, vec![], &runs);
    let out = rf_frame(&ctx, vec![], &runs);
    let m = Metrics::default();
    let p = pills(&out);
    assert_eq!(p.len(), 1, "{p:?}");
    let (pill, fill) = p[0];
    assert_eq!(fill, palette::TAN);
    assert_eq!(outline_at(&out, pill), Some(pill_highlight(palette::TAN)));
    assert_eq!(pill.height(), m.bar);
    assert!(pill.width() >= 40.0, "pill hit area too small: {pill:?}");
    let t = texts(&out);
    assert!(t.iter().all(|(_, r)| !r.intersects(pill.shrink(0.5))), "text painted on the pill: {t:?}");
    assert!(t.iter().any(|(s, _)| s == "RF PATCH"));
    let bar_text = t.iter().find(|(s, _)| s == "RF PATCH").unwrap().1;

    for ev in click_at(pill.center()) {
        rf_frame(&ctx, ev, &runs);
    }
    let id = DetachState::id("rf");
    assert!(DetachState::load(&ctx, id).detached, "click on pill detaches");
    // A new window sits out its first frame, then fades in.
    for _ in 0..40 {
        rf_frame(&ctx, vec![], &runs);
    }
    let out = rf_frame(&ctx, vec![], &runs);
    let hi = pill_highlight(palette::TAN);
    let p = pills(&out);
    let pressed: Vec<Rect> = p.iter().filter(|(_, f)| *f == hi).map(|x| x.0).collect();
    assert_eq!(pressed.len(), 2, "stub and window pills both pressed: {p:?}");
    assert!(pressed.contains(&pill), "same pill rect when detached: bar layout unchanged");
    assert_eq!(outline_at(&out, pill), Some(palette::TAN), "outline swaps to the base colour");
    let t = texts(&out);
    assert!(t.iter().any(|(s, r)| s == "RF PATCH" && *r == bar_text), "title unmoved");
    assert!(t.iter().all(|(_, r)| pressed.iter().all(|pr| !r.intersects(pr.shrink(0.5)))));
    assert!(t.iter().any(|(s, _)| s.contains("RF PATCH: detached")), "stub text shown");
    assert_eq!(t.iter().filter(|(s, _)| s == "CONTENT").count(), 1);
    let win = *pressed.iter().find(|r| **r != pill).unwrap();
    for ev in click_at(win.center()) {
        rf_frame(&ctx, ev, &runs);
    }
    assert!(!DetachState::load(&ctx, id).detached, "window pill attaches");
    let out = rf_frame(&ctx, vec![], &runs);
    assert_eq!(pills(&out), vec![(pill, palette::TAN)]);
    assert!(!texts(&out).iter().any(|(s, _)| s.contains("detached")));
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
