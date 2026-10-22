# 6. Typography and Language

## 6.1 One font, three sizes, two colors

- **Font:** Antonio, a condensed typeface, used for titles, headers and segment labels. Mode buttons and list rows use egui's monospace font (Hack) so that values line up in columns.
- **Sizes:** `title_size` (36), `header_size` (22) and `data_size` (15). Don't add a fourth size. The only exception is that mode pips scale from `data_size`.
- **Colors:** `t.text` for normal text and `t.text_hi` for highlighted text.

## 6.2 Uppercase labels

Buttons, segments and section headers are shown in uppercase; the widgets apply `to_uppercase()` for you. Write labels in normal case in your code, e.g. "Scan". That way the screen-reader label stays natural ("Scan, mode Alt 1").

List rows keep their original case, because they are data such as names and frequencies.

## 6.3 Writing labels

- **Verbs for actions** (SCAN, SAVE, DWELL), **nouns for places** (TUNE, CHANNELS).
- **One or two words.** If a label won't fit, the button is the wrong shape. Make it wider or use a list instead.
- **Don't encode the mode in the label.** The pips show the mode, and the screen-reader label adds the mode name.
- **Same term everywhere** for the same function.

## 6.4 Numbers and units

- **Fixed decimal places** for values in a column (`851.388`, not `851.39` next to `851.3875`).
- **Leading zeros** for counters that scroll (`format!("{v:05}")`), so their width doesn't jitter.
- **Units** after a space in uppercase (`851.388 MHZ`), or once in the column header.
- **Right-align** numeric columns. The `egui::Grid` in the demo shows the pattern.
