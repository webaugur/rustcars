# 11. Checklist and Glossary

## 11.1 Do

- Take every color, size and gap from `LcarsTheme` or `palette`.
- Change thickness at every elbow, and end each bar with a cap.
- Use `mode_button` / `mode_list` for functions that have related variants, and `lcars_button` for navigation.
- Show modes with color **and** pips, and publish mode names to screen readers.
- Keep text at 4.5:1 or more and non-text UI at 3:1 or more; test it.
- Give each task its own screen, and keep controls in place.
- Keep transitions as fades under 1 s; blink no faster than 1 Hz.
- Keep the web interface off by default, bound to localhost, with devices only behind the bridge.
- Ship `OFL.txt` with the font.

## 11.2 Don't

- Add gradients, bevels or shadows to frames.
- Use the same thickness on both sides of a turn, or stretch an elbow.
- Add a fourth font size, a sixth color, or decorative uses of the state colors.
- Put destructive actions on double or triple clicks.
- Move or reflow existing controls when something new appears.
- Rely on color alone, or on hover alone.
- Use `std::time::Instant` in shared UI code.

## 11.3 Glossary

- **Bar:** a straight colored band of the frame. Vertical bars are thick, horizontal bars thin.
- **Cap:** the rounded end of a bar. LCARS's "full stop".
- **Click mode:** one of Inactive / Primary / Alt 1 / Alt 2, chosen by 1–3 clicks or a modifier.
- **Elbow / sweep:** the curved joint between a vertical and a horizontal bar. It is LCARS's signature shape.
- **Frame:** the continuous bent band of bars around content. Here, `LcarsFrame`.
- **Ink:** the label color on a fill, chosen by `ink_on`.
- **OMEGA rule:** the Manifesto's name for the absolute rule that frame thickness changes at every turn.
- **Okudagram:** a nickname for LCARS displays, after designer Michael Okuda.
- **Paint-behind:** reserving a shape slot first and filling it after layout, so chrome can be sized to the content.
- **Pips:** the 1–3 painted dots (·, ∶, ⁖) that show a control's mode.
- **Segment:** a piece of a bar cut out with a gap. It often works as a button.
- **Readout:** a data value display; it is highlighted while being processed.

## 11.4 Credits

- **Design guidance:** Bracer Jack, *LCARS GuideLine* and *The LCARS Manifesto*, republished by lcars-terminal.de.
- **Structure:** inspired by Apple's Macintosh Human Interface Guidelines (1987/1992). No text was copied.
- **Paint-behind technique:** inspired by paxel/dedup-rs (MIT).
- **Font:** Antonio, © 2013 The Antonio Project Authors, SIL OFL 1.1.

Star Trek and LCARS are trademarks and/or copyrights of their respective owners. rustcars is an unofficial fan project and is not affiliated with or endorsed by them.
