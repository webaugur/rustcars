# 5. Color

## 5.1 Palette roles

All colors live in `rustcars::palette` and `Mode::colors()`.

| Constant | Hex | Role |
|---|---|---|
| `BG` | #1B1B1B | Page background, frame interior |
| `SURFACE` | #202020 | List rows, panels |
| `STRIP` | #303030 | Tab strip background |
| `TAN` | #9C805A | Primary frame color ("Tune") |
| `SLATE` | #516D8F | Secondary frame color ("Plan") |
| `OLIVE` | #646F43 | Segments ("Channels") |
| `BRICK` | #945C52 | Alert-adjacent frame ("Meters") |
| `TEAL` | #43736C | Accent ("Levels") |
| `PLUM` | #815E89 | Config / secondary frame |
| `TEXT` | #D9C7A8 | Normal text |
| `TEXT_MUTED` | #9A9A9A | Inactive list rows |

The frame hues come from reference screenshots. They were brightened in the same hue until they reach 3:1 against `BG`.

## 5.2 State colors

| Mode | Fill | Border | Label and pips |
|---|---|---|---|
| Inactive | #1C1C20 | #7A7A82 | #E8E4D8 |
| Primary | #123A1C | #2EB04E | #96E68C |
| Alt1 | #3D2410 | #E08A2E | #FFC27A |
| Alt2 | #2A1040 | #A860D6 | #DCAAFF |

State colors mean *mode*, not *category*. Don't reuse green, orange or purple elsewhere for decoration.

## 5.3 How many colors

From the guideline:

- **One color** plus its tints and shades is always safe.
- **Three colors** is a good place to stop.
- **A fourth or fifth color** must carry a meaning you can name. In rustcars those are the state colors.
- **Six or more** is to be avoided.

For tints and shades of one hue, use `tint(c, t)` rather than adding new hues.

## 5.4 Contrast rules (WCAG 2.2 AA)

- **Text: 4.5:1 or more** against its background, at every size we use.
- **Large text (24 px+ regular):** 3:1 would be allowed, but we keep 4.5:1 anyway.
- **Borders, pips, bars and segments (non-text, SC 1.4.11): 3:1 or more** against what's next to them.
- **Never color alone:** modes also show pips; alerts also change their label.

## 5.5 Verifying

`tests/modes.rs::wcag_contrast` checks every pair and prints a table:

```sh
cargo test --test modes -- --nocapture
```

For your own colors, use `rustcars::contrast(a, b)` in a test:

```rust
assert!(rustcars::contrast(my_label, my_fill) >= 4.5);
```
