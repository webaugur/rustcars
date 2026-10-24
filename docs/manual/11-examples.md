# 11. Example Applications

Six complete desktop applications show the rules in practice. All are built only from rustcars widgets and the theme. They are desktop only; the extra crates they use (`rusqlite`, `directories`, `chrono`) are dev-dependencies, so library users never pull them in.

## 11.1 Scientific calculator

`cargo run --example calculator`. The source is in `examples/calculator/`: `main.rs` holds the UI and `eval.rs` the expression evaluator.

![Calculator](../calculator.png)

- **Function keys** are momentary `mode_button`s. The gesture picks the *variant* of one function:

  | Key | Click (·) | Double (∶) | Triple (⁖) |
  |---|---|---|---|
  | SIN, COS, TAN | sin | asin (inverse) | sinh (hyperbolic) |
  | LN | ln | exp (inverse) | log10 |
  | SQRT | sqrt | ^2 (inverse) | cbrt |
  | PI | π | e | ans |

  Shift and Ctrl work as Alt 1 and Alt 2, and right-clicking a key lists every variant by name.
- **Digits and operators** use `action_button`, which has no modes, so fast typing never merges into a double click.
- **History rows** (`mode_list`):
  - click inserts the result;
  - double-click loads the expression for editing;
  - triple-click plots it. The row stays purple while plotted; a triple click on it removes the plot.
- **Graph:** painted with egui's painter: a grid with a "nice" step, axes, drag to pan, scroll to zoom. Plotted functions use tints of the Alt 2 purple, all at least 4.39:1 against the plot background. Any input containing `x` is previewed live in the text color.
- **Keyboard:** typing goes straight into the expression; Enter evaluates, Backspace deletes, Escape clears.
- **DEG/RAD** is a navigation segment that toggles the angle unit.
- **Evaluator:** a small recursive-descent parser with precedence, right-associative `^`, factorial, implicit multiplication (`2x`), `pi`, `e`, `ans` and `x`. It has unit tests in `eval.rs`.

## 11.2 Diary

`cargo run --example diary [-- --db PATH]`. The source is in `examples/diary/`: `main.rs` holds the UI and `store.rs` the SQLite storage.

![Diary](../diary.png)

- **One focused field.** The NEW ENTRY field takes focus whenever no other widget has it, so you can always just type. Enter saves the entry with the current time.
- **Timestamps** are stored as UTC ISO-8601 (`2026-10-22T12:10:00Z`) and shown in local time.
- **Entry rows** use click modes, and only one row is active at a time:
  - click selects the row and shows Edit and Delete buttons;
  - double-click edits the entry in place;
  - triple-click quotes it into a new entry.
- **Forgiveness:** Delete asks for confirmation, and the last delete can be undone with *Undo delete*, which restores the original id and timestamp.
- **Filtering:**
  - text: a case-insensitive substring match where `%` and `_` count as literal characters;
  - date: a local `YYYY-MM-DD` prefix, so `2026-10` matches a whole month;
  - the *Today* and *All* segments are shortcuts.
- **Storage:**
  - `rusqlite` with the `bundled` feature (a dev-dependency, so library users never pull it in);
  - schema migrations tracked with `PRAGMA user_version`;
  - the database lives in the platform data directory via `directories::ProjectDirs` unless `--db` is given.
- **Tests:** in-memory database tests cover migration, filtering, editing, delete with undo, timestamps, and reopening a file.

## 11.3 Ham RF calculators

`cargo run --example ham_rf`. The source is in `examples/ham_rf/`: `main.rs` holds the UI, `calc.rs` the RF maths and `bands.rs` the US band table. Both modules are pure and unit tested against known values.

![Ham RF: antenna lengths](../ham_rf.png)

Ten tools sit on the tab strip. One **frequency** field in the left frame feeds every tool that needs it, so you type `14.2` once. Under it are two unit pickers (see 11.7).

| Tab | What it does |
|---|---|
| WAVE | Frequency to wavelength, or wavelength to frequency (a picker chooses the direction). Full, half and quarter wave in m, ft, in and cm. |
| ANTENNA | Half-wave dipole (468/f ft, with metric), each leg, quarter-wave vertical and radials, full-wave loop, J-pole and 3-element Yagi starting lengths. Rows copy to the clipboard: click = the selected length unit, double = the other system, triple = the whole line. |
| COAX | Seven common cables (RG-174, RG-58, RG-8X, RG-213, LMR-400, LMR-600, 1/2 in hardline). Loss per 100 ft at the frequency by log-log interpolation, total loss for your length, extra loss from a mismatched load, and the power that reaches the antenna. Click picks a cable, double pins one for comparison. Labeled **approximate**. |
| SWR | Enter SWR, return loss or reflection coefficient; see all of them plus mismatch loss and the power reflected and delivered. |
| POWER | W, mW, dBm, dBW and Vrms (across a load you set, 50 ohm by default) with Vpp and an S-meter estimate; dB to power and voltage ratio, and back. |
| OHM | Ohm's law and power. Enter any two of V, I, R, P: the two most recently edited fields are the inputs. |
| LC | LC resonance. Solve for frequency, inductance or capacitance; also shows the reactance at resonance. |
| MATCH | L-network. Series and shunt reactance and the parts for the low-pass or high-pass form. |
| PATH | Free-space path loss and a simple link budget (power, antenna gains, line loss) to a received level and S-meter estimate. |
| BANDS | US amateur bands with edges and sub-band notes for the frequency you entered. Click a band row to set the frequency to its centre, double for the lower edge, triple for the upper edge. Labeled **informational only**. |

Click modes are used as *variants of one function* here: unit pickers, the direction of a conversion, which quantity to solve for, and the band row's three frequency targets. Nothing destructive sits behind a double or triple click.

![Ham RF: coax](../ham_rf_coax.png)

**Honesty about accuracy.** These are bench rules of thumb. Antenna lengths are starting points to trim with an analyzer. Cable loss uses rounded datasheet figures. The band plan and privileges are a summary: check the current FCC rules (Part 97) before transmitting. Each tool says so on screen.

## 11.4 Timers

`cargo run --example timers`. The source is in `examples/timers/`: `main.rs` holds the UI and `clock.rs` the stopwatch, countdown, clock formats and zones. `clock.rs` never reads the system clock, because callers pass the time in, so its tests are deterministic.

![Timers](../timers.png)

- **Clocks:** a large UTC clock, local time with its offset, day of year and the time to the top of the hour.
- **COPY TIME** is one momentary `mode_button`: click copies a log-style UTC stamp (`2026-10-23 1948Z`), double the local time with offset, triple an ISO 8601 stamp.
- **World clocks** use `mode_list`. Click pins a zone with a 24-hour face, double a 12-hour face and triple an ISO 8601 face. Only one zone is pinned at a time. The zones are fixed standard offsets with no daylight saving, and the screen says so.
- **Stopwatch** with laps (split and total), and a **countdown** with Start/Pause, Clear and `ID 10:00`, a ten minute timer for station identification (FCC 97.119). ADD TIME is one button: click +1 min, double +5, triple +10.
- When the countdown ends the display turns orange and says `TIME!`, so the state is never shown by color alone.

## 11.5 Hex viewer

`cargo run --example hexview [-- FILE]`. With no file it shows the bundled Antonio font as a sample. The source is in `examples/hexview/`: `main.rs` holds the UI and `hex.rs` the row formatting, parsing, search and value decoding.

![Hex viewer](../hexview.png)

- **Rows** are a virtualised `mode_list` inside `ScrollArea::show_rows`, so a 16 MiB file scrolls smoothly. Files larger than that are cut at 16 MiB and the title says so. Set the list's `item_spacing.y` to the same value `mode_list` uses (2 px), or the scroll offsets drift.
- **Three ways to mark a row:** click inspects it (the INSPECTOR shows U8, I8, binary, U16, U32 and F32 in both byte orders), double-click bookmarks it (bookmarks are listed for a quick jump) and triple-click copies its hex bytes to the clipboard.
- **GO TO** takes `496` or `0x1F0`. **FIND** takes hex bytes (`DE AD BE EF`) or quoted text (`"RIFF"`), searches forward from the selection and wraps once.
- Errors are shown in words (`ERROR: ...`) in the status line.

## 11.6 Smith chart

`cargo run --example smith`. The source is in `examples/smith/`: `main.rs` holds the UI and `math.rs` the complex maths, grid geometry, line rotation and stub matching. The SWR, return-loss and mismatch figures reuse `ham_rf`'s `calc.rs` through a `#[path]` module.

![Smith chart](../smith.png)

- **Chart:** painted with egui's painter and scaled to the frame. It shows constant-resistance circles and constant-reactance arcs clipped to the unit circle, with labels on the real axis and the rim. The GRID picker adds the admittance grid (the impedance grid mirrored through the centre).
- **Load:** enter R, X, Z0 (50 ohm by default) and the frequency. The load is plotted with its constant-SWR circle. The panel shows Gamma (magnitude and angle), SWR, return loss, mismatch loss and the normalized impedance and admittance.
- **Line:** a length in wavelengths, degrees or metres (with velocity factor) rotates the load clockwise, toward the generator, along the SWR circle. The arc and the input point `IN` are drawn, and Zin is shown in ohms and normalized. A unit test checks the rotation against the textbook formula Z0 (ZL + jZ0 tan bl) / (Z0 + jZL tan bl).
- **Pointer and markers:** hovering reads off z and Z under the pointer. Clicking places markers through `handle_mode` on the chart's own response: click = primary (green), double = alt 1 (orange), triple = alt 2 (purple), each with its pips. *Clear markers* removes them.
- **Match:** the MATCH picker (off, short stub, open stub) computes a single shunt-stub match. It finds the first point toward the generator where the admittance is 1 + jb, then the stub that cancels jb. The path is drawn in the admittance plane: from yL along the SWR circle to the g = 1 circle, then to the centre.

![Smith chart with admittance grid](../smith_admittance.png)

## 11.7 Patterns worth copying

- **Pickers.** A unit or direction with two or three choices is one `mode_button` showing the current choice. Click selects the first choice, double the second, triple the third. A picker always has a value, so a repeated gesture does not switch it off, and a gesture beyond the number of choices is ignored. A hint line under it names the three gestures. See `pick()` in `ham_rf`. For five or more choices use a row of single-choice buttons (`radio_row()`), not modes.
- **Momentary variants.** Keep a `Mode::Inactive` local, pass it to `mode_button` and act on the result each frame (the calculator's function keys, COPY TIME, ADD TIME).
- **One marked row per meaning.** After `mode_list`, compare the modes with the ones you had, clear the other rows that held the same mode, and then act on the change.
- **Pure core, thin shell.** Put the formulas and parsing in a module with no egui in it and test it with known values. The UI only calls it.

## 11.8 Screenshots

Each example contains a kittest test that renders it headlessly:

```sh
cargo test --example calculator   # writes docs/calculator.png
cargo test --example diary        # writes docs/diary.png
cargo test --example ham_rf       # writes docs/ham_rf.png, ham_rf_coax.png, ham_rf_swr.png, ham_rf_bands.png
cargo test --example timers       # writes docs/timers.png
cargo test --example hexview      # writes docs/hexview.png
cargo test --example smith        # writes docs/smith.png, smith_admittance.png
```

`cargo test --example ham_rf` also renders every tool into `target/ham_rf_tools/` so layout problems are easy to spot. `cargo test --example smith` also renders a smaller window to `target/smith_small.png` to check that the chart scales.
