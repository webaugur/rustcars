# 3. Layout and Frames

## 3.1 Anatomy

An LCARS frame is made of these parts:

- **Vertical bar** (width `bar_thick`, 120 px by default) and **horizontal bar** (height `bar_thin`, 40 px).
- **Elbow (sweep):** the curved joint where the two bars meet. It has an outer radius (about 0.75 × thick) and a concave inner radius (about thin).
- **Cap:** a rounded end that terminates a bar, separated from it by `frame_gap`.
- **Segments:** pieces of a bar cut out with `frame_gap` slits. Segments can act as buttons.

The bars must have **different thicknesses** on either side of a turn. `elbow_shapes` checks this in debug builds.

## 3.2 Elbows

```rust
use rustcars::{Corner, Elbow, palette};
let (tan, bg) = (palette::TAN, palette::BG);
let elbow = Elbow::new(Corner::TL, 120.0, 40.0, tan, bg);
elbow.paint(ui, rect); // paint now
// or get the shapes for paint-behind:
let shapes = rustcars::elbow_shapes(rect, &elbow);
```

How the elbow is built:

- **Defined once, mirrored:** the geometry is written once for the top-left corner (TL) and mirrored for TR, BL and BR.
- **Room for the curve:** the swept block is at least `2 × r_out` in size, because egui shrinks a corner radius to half of the rectangle's shorter side.
- **Background must match:** `bg` must be the interior background color, because it is what carves out the concave inner curve.

## 3.3 LcarsFrame

`LcarsFrame` draws a C-shaped frame (`Side::Left`) or a reversed C (`Side::Right`): top and bottom elbows plus end caps. It is drawn **behind** your content and sized to it:

1. It reserves an empty slot in the paint order with `painter.add(Shape::Noop)`.
2. It lays out your content inside the frame's interior.
3. It fills the slot with `painter.set(...)`, sized to the content's real rectangle.

This technique was inspired by paxel/dedup-rs (MIT).

```rust
use rustcars::{LcarsFrame, Side};
let mut f = LcarsFrame::new(&t, Side::Right, 0.6); // 0.6 = scale
f.top_color = rustcars::palette::PLUM;
f.bottom_color = rustcars::palette::BRICK;
f.min_height = ui.available_height(); // fill the column
let (_inner, frame_rect) = f.show(ui, &t, |ui| { /* content */ });
```

`frame_rect` is the frame's outer rectangle. Use it to cut segment buttons into the vertical bar, as the demo does.

## 3.4 Spacing grid

Only two spacing values exist:

- **`frame_gap` (4 px):** between segments, and between bars and caps.
- **`main_gap` (16 px):** between major regions, and between the frame and its content.

Everything else is derived from these. The only exceptions are text and the top row of navigation segments.

## 3.5 Screen composition

A typical screen:

- **Top:** `egui::Panel::top` holds the tab/navigation strip and the title.
- **Body:** one or two `LcarsFrame`s side by side inside the `CentralPanel`, split on the grid with a `main_gap` between them.
- **Segments:** the left frame's vertical bar holds navigation or mode segments.

Use `ui.scope_builder(UiBuilder::new().max_rect(r), ..)` to place a frame in a computed rectangle. egui's `Margin` type is limited to `i8` (127 px), which is too small for LCARS bars.

## 3.6 One task per screen

Each screen does one job, such as *Tune*, *Plan* or *Channels*. When the user switches tasks:

- switch the whole screen;
- don't swap controls in place;
- never let new content push existing controls to a new position.

If a control has to appear temporarily, reserve its space in advance or show it in a new screen.

## 3.7 Group boxes and detaching

A group box is a thin titled elbow around one group of controls: a 16 px bar with the title knocked out, a rail down the left, and an end pill on the right.

```rust
rustcars::group_box(ui, "Radio", rustcars::palette::SLATE, |ui| { /* controls */ });
```

Give a group box a name with `.detachable(...)` and its end pill becomes a detach/attach toggle:

```rust
rustcars::GroupBox::new("RF patch", rustcars::palette::TAN)
    .detachable("rf-patch")
    .show(ui, |ui| { /* controls */ });
```

- The pill has no text. It keeps the round elbow-end shape and the same size in both states, so the bar never shifts.
- **Attached (raised):** the pill is the elbow's colour with a 1.5 px outline in a lighter highlight of that colour.
- **Detached (pressed in):** the colours swap: highlight fill, outline in the elbow's colour, plus a thin inner line.
- Hover the pill to read what a click does ("Detach: open RF PATCH in its own window." or "Attach: bring RF PATCH back here…"). Screen readers get the same words.
- Detaching moves the elbow and everything in it to its own window, which shows the same elbow with a pressed-in pill.
- In the original place the elbow stays, with the text "RF PATCH: detached, in its own window. Click the end pill to bring it back."
- Clicking either pill, or closing the window, puts the group back.
- The detached window has no title bar or border of its own: the elbow is its frame, with a thin outline in the elbow colour. **Drag the pill to move the window**; a click without moving still puts it back. The move is done by your window manager, so its own snapping (full, half or quarter screen) works as usual.
- Resize it from the small grip in the bottom-right corner, or with your window manager's usual resize gestures. The window remembers its size and place, and is kept on screen.
- Some window systems ignore move requests from applications. There the pill still docks the window, but dragging it does nothing; use your window manager's own move gesture (often Super + drag) instead.
- Group boxes nest. A detached group inside a page that is itself detached stays in its own window; the attached ones travel with the page.
- The contents are drawn only in the place they are shown. In a detached window egui may draw a frame twice, so the contents closure can run more than once there: pass a closure that can be called again (`FnMut`). `show` returns `Option`: `None` in a frame where the window could not be drawn.
- If the window system cannot open the window, the stub says so in plain text ("RF PATCH: detached window failed: …; click the pill to bring it back") and the next frame tries again.

Where a group is lives in egui memory under its name (`rustcars::DetachState`). An app that wants it remembered between runs can read and write it with `DetachState::load` and `store`.

For a frame that is not a group box, use the parts directly: `group::detach_pill` paints the toggle in any end cap, and `group::detached_window` shows contents in their own window.
