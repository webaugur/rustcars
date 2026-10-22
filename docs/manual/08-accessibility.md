# 8. Keyboard, Accessibility and Screen Readers

## 8.1 Keyboard

| Key | Effect |
|---|---|
| Tab / Shift+Tab | Move focus between controls |
| Enter / Space | Primary (single click) |
| Shift+Enter, Shift+click | Alt 1, applied immediately |
| Ctrl/Cmd+Enter, Ctrl/Cmd+click | Alt 2, applied immediately |
| Right-click / long-press | Context menu listing every mode by name |

The modifier shortcuts work for people who can't double- or triple-click reliably. They commit at once, with no timing involved.

## 8.2 Context menu

Every mode control has a context menu with *Off*, *Select (click)*, *Alt 1 (double-click / Shift)* and *Alt 2 (triple-click / Ctrl)*. The menu doubles as on-screen documentation of the gestures.

## 8.3 Screen readers (AccessKit)

`mode_button` and `mode_list` rows publish a label of the form *"Scan, mode Alt 1"* through egui's AccessKit integration (`Response::widget_info`). If you build your own controls, do the same:

```rust
let text = format!("{label}, mode {}", mode.short_name());
let enabled = ui.is_enabled();
resp.widget_info(|| {
    egui::WidgetInfo::labeled(egui::Role::Button, enabled, &text)
});
```

## 8.4 Never color alone

Every state shows its color **and** a second cue:

- **Modes:** pips (·, ∶, ⁖), painted with `paint_pips`. They are drawn shapes rather than font characters, because the triangle character ⁖ isn't in any of the bundled fonts.
- **Highlighted values:** shown with weight or position as well as color.
- **Alerts:** a changed label as well as color.

## 8.5 Hit sizes

- **Touch targets:** at least 44 × 44 px; desktop list rows at least 28 px tall.
- **Thin bars:** a frame bar thinner than a finger shouldn't be a button (Manifesto). Place a separate segment there instead.
