# 1. Design Philosophy and Principles

rustcars brings the LCARS look to ordinary egui applications. LCARS is the fictional computer interface from *Star Trek*, designed by Michael Okuda. Looking like LCARS is only half the job, though. The interface also has to *work* for people doing real tasks. This chapter sets out the principles behind every other chapter. They are adapted from Bracer Jack's *LCARS GuideLine* and *LCARS Manifesto*, and they take inspiration from the structure of classic desktop interface guidelines such as Apple's Macintosh Human Interface Guidelines (1987/1992). The wording and the rules here are our own.

## 1.1 Metaphor: the frame is the instrument panel

An LCARS screen is a **console**. Colored bars bend around the content like the edge of an instrument panel, and pieces of those bars act as controls. The metaphor tells users two things:

- **Inside the frame** is the information they are working on.
- **On the frame** are the controls that act on it.

Keep that split. Don't scatter controls across the content area when a frame segment could hold them. And don't use frame color merely to label a category; the Manifesto calls a frame that does nothing but label "already a 20% sin".

## 1.2 Direct manipulation, see-and-point

Users should act on what they can see:

- Every function a screen offers appears as a visible button, list row or segment.
- The user points at it (mouse, touch or keyboard focus) and clicks.
- The mode a control is in is visible right on the control, as its color plus pips.
- Nothing important should be reachable only through a hidden gesture. The context menu (Chapter 8) is a *second* path to the same modes, never the only one.

## 1.3 Consistency

The same thing looks and behaves the same everywhere:

- **Two spacing constants:** `frame_gap` and `main_gap`.
- **Three font sizes:** title, header, data.
- **Two text colors:** normal and highlight.
- **One set of state colors:** gray, green, orange, purple.
- **One click-mode grammar** for every multi-mode control.

`LcarsTheme` and `palette` hold these values. Never write a color or a pixel size directly in application code.

## 1.4 User control

The user starts actions; the interface doesn't act on its own initiative.

- Rearranging the screen, starting a scan or changing a mode happens only because the user asked.
- Background activity, such as a scanner stepping through channels, is shown in readouts but never takes over the controls.

## 1.5 Feedback and dialog

Every action gets an immediate, visible response:

- **On press:** the button shade changes while it is pressed (`tint(base, -0.25)`).
- **On mode change:** the new state color and pip count appear.
- **On data change:** the readout highlights the value being processed (`readout(.., highlight = true)`).
- **When something is slow:** show progress in a readout (Chapter 7) rather than freezing.

## 1.6 Forgiveness

People make mistakes, so make them cheap:

- Repeating a mode gesture turns the control **off** again (`Mode::after_gesture`).
- Destructive actions ask for confirmation, and where possible offer **undo** instead (see the diary example, Chapter 12).
- Never put a destructive control directly next to a frequently used one.

## 1.7 Perceived stability: controls never move

The Manifesto is explicit about this. A real interface must not "reconfigure" itself under the user's hands.

- A new task gets **a new screen**.
- New controls may appear, but they must never shift existing ones.
- Transitions are short fades, under a second (Chapter 7).

People build muscle memory for where a control lives. Moving it breaks that memory.

## 1.8 Aesthetic integrity

Keep the look clean, flat and coherent:

- No gradients, bevels or shadows on the frame.
- At most five meaningful colors.
- Thickness changes at every turn of the frame (the "OMEGA" rule, see the Glossary).
- Decorative animation (scrolling numbers, blinkies) is allowed only when it never gets in the way of the task.

## 1.9 Modelessness versus click modes

Classic guidelines warn against *modes*: states where the same action does different things. rustcars uses **click modes** on purpose, but keeps them safe:

- **Local:** a mode belongs to one control and one function only. Nothing changes globally.
- **Visible:** the mode is shown on the control by color *and* pips.
- **Few:** at most three modes plus Off; four or more clicks clamp to the third.
- **Reversible:** repeating the gesture turns the control off.
- **Reachable without timing:** Shift/Ctrl modifiers and the context menu reach every mode.

If a function doesn't naturally have related variants, give it a plain single mode. Don't invent alternates just to fill the slots.
