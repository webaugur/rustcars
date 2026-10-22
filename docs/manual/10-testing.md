# 10. Testing

## 10.1 Snapshot tests (kittest)

`tests/preview.rs` renders the demo headlessly with `egui_kittest` and wgpu:

```rust
let mut harness = egui_kittest::Harness::builder()
    .with_size(egui::vec2(1280.0, 800.0))
    .wgpu()
    .build_ui(move |ui| rustcars::demo(ui, &t, &mut st));
harness.run_steps(4);
harness.snapshot("lcars_demo"); // tests/snapshots/lcars_demo.png
```

- **First run:** run `UPDATE_SNAPSHOTS=1 cargo test` to accept a new snapshot.
- **Without a GPU:** install Mesa's software Vulkan driver (lavapipe), e.g. `apt install mesa-vulkan-drivers`.
- **Exact pixels:** snapshots are compared pixel for pixel, so a real GPU may differ slightly from lavapipe. Keep one reference machine, or raise the threshold with `SnapshotOptions`.

## 10.2 Contrast tests

Add every new color pair to a test that uses `rustcars::contrast` (Chapter 5). The thresholds are 4.5 for text and 3.0 for non-text.

## 10.3 Behavior tests

The click-mode rules are pure functions that are easy to test:

```rust
use rustcars::Mode;
// clicks beyond three clamp to Alt2
assert_eq!(Mode::from_clicks(5), Mode::Alt2);
// repeating the current gesture turns the control off
assert_eq!(Mode::Alt1.after_gesture(2), Mode::Inactive);
```

## 10.4 Before you commit

`scripts/check.sh` runs:

- `cargo fmt --check`;
- clippy (`-D warnings`) for the desktop, wasm and server builds;
- all tests;
- a check that the default build stays free of web dependencies.
