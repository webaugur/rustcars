# 4. Controls

## 4.1 Buttons and click modes

`mode_button(ui, &t, label, &mut mode, size)` draws a rectangle with a small rounding (4 px), a dark fill tinted by state, a 1 px border and an uppercase monospace label.

| Gesture | Keyboard / alternative | Mode | Color | Pips |
|---|---|---|---|---|
| (none) | context menu: Off | `Inactive` | gray | none |
| single click | Enter / Space | `Primary` | green | · |
| double click | Shift+click, Shift+Enter | `Alt1` | orange | ∶ |
| triple click | Ctrl/Cmd+click, Ctrl/Cmd+Enter | `Alt2` | purple | ⁖ |

How a click sequence is resolved (`handle_mode`):

- **Committed once.** egui's `clicked`, `double_clicked` and `triple_clicked` only raise a pending click count. The gesture is applied once the double-click window (0.3 s by default) closes, or immediately on a triple click. So a triple click never also fires the single or double action.
- **Clamped at three.** Four or more clicks count as a triple, and further clicks in the same burst are ignored.
- **Same gesture turns it off.** Repeating the gesture of the current mode switches the control to `Inactive`. A different gesture switches directly to that mode (`Mode::after_gesture`).

### When to use each mode

- **Primary (single click):** the ordinary, most frequent meaning, e.g. *Scan*, *Select channel*, *sin*.
- **Alt 1 (double click):** a closely related variant of the same function, e.g. *Scan priority only*, or *asin* (the inverse function).
- **Alt 2 (triple click):** the rarer second variant, e.g. *Scan and record*, or *sinh*.
- **Single-meaning functions:** use `lcars_button` (no modes) or ignore `Alt1`/`Alt2`. Never invent alternates just to fill the slots, and never give the alternates unrelated meanings.
- **Destructive actions:** never put one on a double or triple click, because accidental multi-clicks happen.

### Toggle buttons

`toggle_button(ui, &t, label, &mut on, hue, size)` draws an on/off button. A click flips `on` and the response reports `changed()`.

- **Toggles have a thin border, momentary buttons don't.** A toggle (Live, Data, Run) has a 1.5 px outline. A momentary button (Today, Fit, +) is a solid filled rectangle with no outline. The border tells you at a glance which buttons hold a state.
- **On:** filled in the hue with a light border. **Off:** dark fill with a border in the hue. Borders reach at least 3:1 against the fill, labels at least 4.5:1.
- **State is written, not just coloured.** The label starts with ● when on and ○ when off, e.g. `● LIVE`, `○ DATA`. The marker is painted as a circle because the LCARS font has no ●/○ characters. `paint_toggle_marker_label` draws it for custom widgets, and `toggle_label(label, on)` gives the text form for screen readers and logs.
- Screen readers announce it as a checkbox with its on/off state.
- The stopwatch and countdown **Run** buttons in `examples/timers` are toggles.

## 4.2 Lists

`mode_list(ui, &t, id_salt, &mut rows, row_h)` takes a `&mut [(String, Mode)]`. Each row behaves exactly like a mode button:

- Inactive rows are flat (`palette::SURFACE`) with muted text.
- Active rows get the state fill, a 1 px border and pips.
- Each row has a screen-reader label like "1.031 Dalton, mode Alt 2".

Keep row heights at 28 px or more on desktop, and 44 px for touch (Chapter 8).

## 4.3 Readouts

`readout(ui, &t, value, highlight)` shows a data value in the data size. `highlight = true` uses `t.text_hi` to mark the value currently being processed. Put units in the same string and keep their case consistent (Chapter 6).

## 4.4 Tabs and navigation segments

`lcars_button(ui, &t, label, base, size, cap, enabled)` draws a solid frame segment. It is used for:

- the top tab strip (Tune, Plan, Channels, ...);
- segments cut into a frame bar.

`Cap::Left`, `Cap::Right` and `Cap::Both` round the ends; `Cap::None` gives a plain segment. Label color is chosen automatically with `ink_on`. Segments are for **navigation**, so give them a single meaning and no click modes.

## 4.5 Disabled state

For segments, pass `enabled = false` to `lcars_button`. It is drawn as `tint(base, -0.6)` and doesn't respond to clicks. WCAG exempts disabled controls from contrast requirements, but the label must still be readable.

Prefer hiding a control that is *never* available on a screen, rather than showing it permanently disabled.

## 4.6 Focus

Every control responds to the keyboard:

- Tab moves between controls.
- A focused control shows a 1–2 px ring in `t.text`.
- Enter or Space triggers the primary action.

Don't remove the focus ring.
