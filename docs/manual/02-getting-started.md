# 2. Getting Started

## 2.1 Add the crate

```toml
[dependencies]
rustcars = { git = "https://github.com/webaugur/rustcars" }
egui = "0.37"
eframe = "0.37"
```

The default features are desktop only. The `web` and `web-server` features are opt-in (Chapter 9).

## 2.2 Fonts and license obligations

`install_lcars_font(ctx)` loads the bundled **Antonio** typeface (`rustcars::ANTONIO_TTF`) as the first proportional font. egui's built-in fonts stay installed as fallbacks.

Antonio is licensed under the **SIL Open Font License 1.1**. If you ship the font, you must:

- ship `assets/fonts/OFL.txt` and the copyright notice from `assets/fonts/FONTS.md` alongside it;
- never sell the font file on its own;
- keep any modified version of the font under the OFL.

To use a different font, call `install_font(ctx, include_bytes!("my.ttf"))`, and check its license before bundling it.

## 2.3 Apply the theme

```rust
let theme = rustcars::LcarsTheme::default();
rustcars::install_lcars_font(ctx);
rustcars::apply_theme(ctx, &theme);
```

`apply_theme` does the following:

- forces the dark theme;
- sets the three text styles (Heading, Button, Body/Small/Monospace);
- sets the spacing constants;
- removes shadows and strokes;
- fills panels with `palette::BG`.

Call it once at startup, for example in your `App::new`.

## 2.4 A minimal application

The complete program is `examples/minimal.rs` (`cargo run --example minimal`). The interesting part:

```rust
impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _f: &mut eframe::Frame) {
        let t = self.theme;
        let bg = egui::Frame::new().fill(t.bg).inner_margin(t.frame_gap);
        egui::CentralPanel::default().frame(bg).show(ui, |ui| {
            let mut frame = LcarsFrame::new(&t, Side::Left, 1.0);
            frame.min_height = ui.available_height();
            frame.show(ui, &t, |ui| self.content(ui, &t));
        });
    }
}

impl App {
    fn content(&mut self, ui: &mut egui::Ui, t: &LcarsTheme) {
        ui.horizontal(|ui| {
            let seg = egui::vec2(110.0, t.bar_thin);
            lcars_button(ui, t, "Home", t.primary, seg, Cap::Left, true);
            let size = egui::vec2(96.0, 56.0);
            mode_button(ui, t, "Scan", &mut self.scan, size);
        });
        mode_list(ui, t, "channels", &mut self.rows, 28.0);
        let busy = self.scan != Mode::Inactive;
        readout(ui, t, "851.388 MHZ", busy);
    }
}
```

The application owns the state: each `Mode` and the list rows live in your struct. The widgets only read and update them.

![The rustcars demo](../preview.png)
