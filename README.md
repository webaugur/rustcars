# rustcars

LCARS-style UI widgets and a demo for [egui](https://github.com/emilk/egui) 0.37 in Rust. Included:
- elbows for all four corners
- paint-behind frames that size themselves to their content
- pill buttons whose text color is chosen automatically for contrast
- data readouts
- a theme limited to the three font sizes and two spacing values the guideline calls for
- the Antonio font, bundled

Function buttons and list rows use click-count modes: click selects (green), double-click picks alternate 1 (orange), triple-click picks alternate 2 (purple). They are drawn in a muted LCARS palette that meets WCAG 2.2 AA (see `AGENTS.md` §0).

The repo also contains [`AGENTS.md`](AGENTS.md), a set of design and implementation rules for coding agents building LCARS UIs with egui.

![preview](docs/preview.png)

## Run

```sh
cargo run            # opens the demo window (eframe)
```

## Test / re-render the preview

`tests/preview.rs` renders the demo headlessly with `egui_kittest` (wgpu). It writes `docs/preview.png` and compares the result against the snapshot in `tests/snapshots/lcars_demo.png`.

```sh
cargo test                       # renders + compares snapshot
UPDATE_SNAPSHOTS=1 cargo test    # accept a new snapshot
```

The test needs a Vulkan or GL device. On a machine without a GPU, install Mesa's software driver (lavapipe), e.g. `sudo apt install mesa-vulkan-drivers libvulkan1` on Debian/Ubuntu.

```sh
cargo clippy --all-targets -- -D warnings
```

## Usage

```rust
let t = rustcars::LcarsTheme::default();
rustcars::install_lcars_font(ctx);
rustcars::apply_theme(ctx, &t);
// inside a Ui:
rustcars::LcarsFrame::new(&t, rustcars::Side::Left, 1.0).show(ui, &t, |ui| {
    rustcars::lcars_button(ui, &t, "Engage", t.primary, egui::vec2(150.0, 44.0), rustcars::Cap::Both, true);
});
```

## Credits

- **Design guidance:** "LCARS GuideLine" and "The LCARS Manifesto" by Bracer Jack (originally bracercom.com), republished by [lcars-terminal.de](http://www.lcars-terminal.de/tutorial/guideline.htm). `AGENTS.md` summarizes them with attribution.
- **Paint-behind elbow technique:** inspired by [paxel/dedup-rs](https://github.com/paxel/dedup-rs) (MIT). The code here is original.
- **Font:** [Antonio](https://github.com/googlefonts/antonioFont) by Vernon Adams, © 2013 The Antonio Project Authors, SIL Open Font License 1.1. See `assets/fonts/OFL.txt` and `assets/fonts/FONTS.md`.

## Disclaimer

Star Trek and LCARS are trademarks and/or copyrights of their respective owners (Paramount/CBS). LCARS was designed by Michael Okuda. This is an unofficial fan project and is not affiliated with or endorsed by those owners.

## License

The code is under the MIT License (see [LICENSE](LICENSE)). The bundled font is under the OFL 1.1.
