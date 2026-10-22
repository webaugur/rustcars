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
    // every active mode has a non-color indicator
    for m in [Mode::Primary, Mode::Alt1, Mode::Alt2] {
        assert!(!m.tag().is_empty());
    }
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
