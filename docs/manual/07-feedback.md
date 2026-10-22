# 7. Feedback, Motion and Sound

## 7.1 Transitions

When the screen changes:

- **Fade out, then fade in.** Never crossfade.
- **Keep it under one second** in total; about 120 ms out and 150 ms in is plenty. `ctx.animate_bool_with_time(id, visible, 0.15)` gives a 0–1 value to use as opacity.
- **Animation is optional.** Make it switchable for users who prefer reduced motion.

## 7.2 Blinking and alerts

- **Rate:** blink alerts at about 1 Hz. Never flash between 3 and 50 Hz (photosensitivity, WCAG 2.3.1).
- **More than color:** an alert also changes its label (e.g. "ALERT") or adds an icon.
- **Where alerts appear:** in a fixed place on the frame, never as a pop-up that covers controls.

## 7.3 Progress

Show long operations in a readout with a value or percentage that the user can read, and highlight it with `text_hi`. Keep the rest of the screen usable.

Refresh at a modest rate, e.g. `ctx.request_repaint_after(Duration::from_millis(250))`, rather than every frame.

## 7.4 Decorative motion

Scrolling numbers are allowed as decoration, but keep them out of the user's way:

- generate them in code;
- give each element its own period, so they never visibly loop together;
- drive them from `ui.input(|i| i.time)`, never `std::time::Instant`, which panics on wasm.

## 7.5 Sound

Sound is optional and must never be the only signal. If you use it, keep it short and quiet, and give the user a mute control.
