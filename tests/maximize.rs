//! A maximized detached window: the pill restores it before the WM drag,
//! and its maximized size is never saved (GNOME auto-maximizes a window
//! reopened at nearly monitor size).
use egui::{Rect, ViewportCommand, pos2, vec2};
use rustcars::group::{SAVED_MAX_SHARE, drag_cmds, geometry_to_save};

fn info(max: bool) -> egui::ViewportInfo {
    egui::ViewportInfo {
        maximized: Some(max),
        inner_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1920.0, 1050.0))),
        outer_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1920.0, 1080.0))),
        monitor_size: Some(vec2(1920.0, 1080.0)),
        ..Default::default()
    }
}

#[test]
fn a_maximized_window_is_restored_before_the_drag() {
    assert_eq!(drag_cmds(&info(true)), vec![ViewportCommand::Maximized(false), ViewportCommand::StartDrag]);
    assert_eq!(drag_cmds(&info(false)), vec![ViewportCommand::StartDrag]);
    let mut f = info(false);
    f.fullscreen = Some(true);
    assert_eq!(drag_cmds(&f), vec![ViewportCommand::Fullscreen(false), ViewportCommand::StartDrag]);
}

#[test]
fn a_maximized_size_is_never_saved_and_saved_sizes_stay_under_auto_maximize() {
    assert_eq!(geometry_to_save(&info(true)), (None, None));
    let s = geometry_to_save(&info(false)).0.unwrap();
    assert!(s.x <= 1920.0 * SAVED_MAX_SHARE + 0.5 && s.y <= 1080.0 * SAVED_MAX_SHARE + 0.5, "{s:?}");
}
