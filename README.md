# tui — a terminal user interface library

A buffer of grapheme clusters, a diffing renderer, ratatui's constraint
layout, width-aware text, a styling layer, twenty-one widgets and widget-like
pieces, a mouse router, and the application loop that joins all of it to a real
terminal. Nothing below the
loop touches a terminal: everything else is computation against an
in-memory buffer, and the output of a frame is one `bytes`. That is what
makes it testable, and it is the whole design.

Stage 1 was the buffer, the layout solver, width-aware text and five
widgets. **Stage 2 and stage 3 are both done**: the styling layer (`Theme`,
styled spans), the primitives `apps/git/design.md` names (a collapsible
outline, a synced jump list, a which-key overlay and the transient menu
built on it, an addressable diff view, a persistent footer, plus `Viewport`
and `Scrollbar` in support of the first two), and `TUI_app.Loop`, which reads
real keys through `lib/term.m31` and redraws through this library's own
renderer. **Stage 4 is also done**: the dashboard widgets (`Gauge`, `Sparkline`,
`BarChart`, `Chart`), `Spinner`, the input widgets (`TextInput`, `TextArea`),
`Tabs`, and mouse support end to end: a `Pointer` that turns SGR reports into
clicks, drags and wheel turns, a `Router` that sends each to the region under
it, `handle_mouse` on every interactive widget, and an opt-in timer, mouse and
paste mode on `TUI_app.Loop`. See "Left deliberately undone" below for what
remains.

    M31_ROOT=/path/to/m31 bash tests/test.sh                              # tests, then the benchmark
    M31_ROOT=/path/to/m31 bash scripts/build.sh examples/demo.m31 -o /tmp/d    # a static screen; run /tmp/d
    M31_ROOT=/path/to/m31 bash scripts/build.sh examples/browse.m31 -o /tmp/b  # an interactive one; run /tmp/b
    M31_ROOT=/path/to/m31 bash scripts/build.sh examples/widgets.m31 -o /tmp/w # every stage 4 widget, mouse and timer

**This library is not part of the standard library.** The compiler knows
nothing about it, nothing was added to `src/stdlib.rs`, and nothing was put
in `lib/`. It is an ordinary library: with m31 v0.2.0 or later a program
depends on it from its own `deps` file and imports each module by path. This
version (0.7.0) is built and tested against m31 v0.4.0; `RENAMES.md` lists what
a program moving from 0.4.0 or from m31 0.3.x has to change.

    name myapp
    version 0.1.0
    tui https://github.com/qrazil/tui <ref>

    import tui.app.TUI_app;

(`tui path ../tui` uses a checkout in place.) The modules sit in three
folders, `core/`, `widgets/` and `app/`, so a program's import is
`tui.<folder>.<module>`; the module is still called by its own name
(`TUI_app.Loop`). This repository's own `deps` file is what marks the root, so
the library's modules, `examples/` and `tests/` import each other by
`<folder>.<module>` (`import core.TUI_buffer;`). What the language used to
force instead is written up in `docs/FRICTION.md` §1.

---

## 0.9.0: the renderer for smooth output

* `TUI_diff.diff_scrolled` / `diff_scrolled_in_place`: a view that moved by a few lines is scrolled with a terminal
  region (`CSI t;b r`, line feeds / reverse index, `CSI r`) and only the rows that opened up are written. It matches
  whole identical rows only, so a vertical split or a sidebar that did not move falls back to `diff`.
  `Buffer.scrolled`, `scroll_rows`, `copy`, `is_row_like`, `is_blank_row` support it.
* `TUI_app.Loop`: synchronized output (DEC 2026) asked with DECRQM and used only when the terminal answers
  (`should_sync`, `is_sync`); the real cursor and its DECSCUSR shape (`cursor`, `should_manage_cursor`); focus reports
  (`should_report_focus`, `has_focus`); `is_input_queued` while a callback runs; `frame_count`, `last_frame_bytes`,
  `last_frame_us`; `animate_ms` may be changed by a callback and is read every pass; the clock is `timer.monotonic_ns`
  (it was the wall clock). Needs m31 0.5.0 (`timer`).

## The modules

Twenty-eight, in three folders (see "Layout" below). They are all prefixed `TUI_` because module
names are globally unique and every unprefixed name a UI library wants is
either taken by `lib/` or likely to collide with a user's own file — see
`docs/FRICTION.md` §1. `TUI_app` is the one exception worth naming: it is the only
module that imports `lib/term.m31` and touches raw mode, a real read and a
real write; every other module below it is still pure computation against a
buffer.

| module | folder | what is in it |
|---|---|---|
| `TUI_style` | `core/` | `Colour`, `Style`, `Depth`, the attribute bits, the SGR escapes, and `Theme` |
| `TUI_geometry` | `core/` | `Rect`, `Constraint`, `Layout`, and the constraint solver |
| `TUI_text` | `core/` | `Align`, `wrap`, `truncate`, `pad`, `fit` — all in terminal columns |
| `TUI_buffer` | `core/` | `Cell` and `Buffer` |
| `TUI_widget` | `core/` | the `Widget` interface, `Block`, `Paragraph`, `Span` and `Line` |
| `TUI_scroll` | `core/` | `Viewport` (a scroll offset and its clamping/reveal math) and `Scrollbar` |
| `TUI_eighths` | `core/` | eighth-block bar maths: `of_ratio`, `scaled`, `vertical`, `horizontal`, `bounds_of` |
| `TUI_scale` | `core/` | `nice_ticks` and `Scale`: one mapping that places both tick marks and their labels |
| `TUI_clip` | `core/` | `put_str` and `paint` into a clip rectangle, so a widget cannot draw outside its area |
| `TUI_word` | `core/` | word-boundary motion over grapheme clusters, shared by the two text widgets |
| `TUI_syntax` | `core/` | `language_of`, `classify`, `class_style`, `spans`, `draw`, `Highlighter` — a lexical highlighter and the styles to draw it in |
| `TUI_mouse` | `core/` | `Gesture`, `Pointer`, `Region`, `Router` — mouse reports to clicks, drags and hovers, routed to widgets |
| `TUI_list_table` | `widgets/` | `ListView` and `Table` |
| `TUI_outline` | `widgets/` | `Node`, `Outline` — the collapsible primary-view document |
| `TUI_jump_list` | `widgets/` | `Entry`, `JumpList` — a flat index synced two ways to an `Outline` |
| `TUI_menu` | `widgets/` | `Popup`, `WhichKey`, `Switch`, `Transient` — one base, two overlays |
| `TUI_diff_view` | `widgets/` | `Kind`, `Line`, `Hunk`, `DiffView` — a diff with a cursor addressable to the line, optionally syntax-coloured (`language`), hunks that fold (`Hunk.is_collapsed`, `DiffView.toggle_hunk`) |
| `TUI_footer` | `widgets/` | `split`, `Footer` — a screen band reserved regardless of the main view |
| `TUI_gauge` | `widgets/` | `Gauge` — a ratio as a filled bar with a label, in eighths of a cell |
| `TUI_sparkline` | `widgets/` | `Sparkline` — a series as a row of vertical eighth-blocks |
| `TUI_barchart` | `widgets/` | `Bar`, `BarGroup`, `BarChart` — vertical or horizontal, grouped, with value labels |
| `TUI_chart` | `widgets/` | `Dataset`, `Axis`, `Chart` — line, scatter and bar over braille, dot or block markers, with a legend and a hover readout |
| `TUI_spinner` | `widgets/` | `Spinner` and its frame sets, advanced by the loop's timer |
| `TUI_input` | `widgets/` | `TextInput` — a one-line editor: selection, kill ring, undo/redo, paste, validation, scrolling |
| `TUI_textarea` | `widgets/` | `TextArea` — a multi-line editor: selection, wrapping, line numbers, undo/redo, wheel scroll |
| `TUI_tabs` | `widgets/` | `Tab`, `Tabs` — a tab bar with overflow scrolling, selection and close requests |
| `TUI_diff` | `app/` | `full`, `diff`, and the small control sequences |
| `TUI_app` | `app/` | `Action`, `Handler`, `Ticker`, `MouseHandler`, `Loop` — the application loop, over `lib/term.m31` |

Each file's header comment is its reference documentation.

---

## The shape of a program

```c
import tui.app.TUI_diff;
import tui.core.TUI_buffer;
import tui.core.TUI_geometry;
import tui.core.TUI_style;
import tui.core.TUI_text;
import tui.core.TUI_widget;
import tui.widgets.TUI_list_table;

TUI_buffer.Buffer screen = TUI_buffer.Buffer.sized(80, 24);

List<TUI_geometry.Rect> band = TUI_geometry.rows(screen.area(),
    [TUI_geometry.Constraint.Length(3), TUI_geometry.Constraint.Fill(1)]);

TUI_widget.Block panel = TUI_widget.Block(title: "files",
    borders: TUI_widget.Borders.Rounded);
panel.render(screen, band[1]);
TUI_list_table.ListView(names, selected: 3, highlight_symbol: "> ")
    .render(screen, panel.inner(band[1]));

bytes frame = TUI_diff.full(screen, TUI_style.Depth.Ansi256);   // one write
```

A widget is handed the rectangle it draws in; it holds no position of its
own. A block and its content are two widgets, composed by the caller with
`block.inner(area)` — there is no `.block(..)` on every widget, so every
widget stays independent of `Block` and a user's widget is no harder to use
than a shipped one.

---

## A cell is a grapheme cluster

Not a code point. `é` written as `e` + U+0301 is one cell, `👍🏽` is one cell,
a flag is one cell.

A cluster two columns wide — a CJK ideograph, an emoji — occupies **two**
cells: the cluster, and a *continuation* whose symbol is empty and whose
width is 0. The renderer never writes a continuation, and every write keeps
the invariant that a continuation has a two-wide cell immediately to its
left. Overwriting either half of a pair turns the other into a space. That
is where terminal libraries have bugs, and it is checked from both
directions in `tests/tests.m31`.

---

## The layout rules

`TUI_geometry.resolve(total, constraints)` answers one size per constraint, and
**the sizes always sum to exactly `total` and none is negative** — so the
parts tile the whole with no gap and no overlap. That invariant is checked
against 2 000 random constraint sets from a seeded generator, horizontally
and vertically, in `tests/tests.m31`.

Each constraint states a preferred size, clamped into `0..total`:
`Length(v)` is `v`, `Percentage(p)` is `total * p / 100`, `Ratio(a, b)` is
`total * a / b`, `Min(v)` and `Max(v)` are `v`, and `Fill(w)` is 0.

If they come to **less** than the total, the surplus goes to the `Fill`s by
weight; failing that to the `Min`s equally; failing that to the last
constraint. If they come to **more**, the deficit is taken back in five
rounds — `Fill`, then `Max`, then `Percentage` and `Ratio`, then `Min`, then
`Length` — and within a round it is shared in proportion to what each
segment has to give. Both directions use largest-remainder rounding, in
whole cells, with ties to the earlier segment.

The full statement, including why `[Max(30), Max(30)]` in 100 cells gives 30
and 70, is the header comment of `TUI_geometry.m31`.

---

## What the renderer decides

`TUI_diff.diff(prev, next, depth)` writes a cell when its symbol, style or
width changed. Between two changed cells it either jumps (`\x1b[nC`) or
walks (writes the unchanged cells again), whichever is fewer bytes — and
only walks when the run is already in the pen's style. Row changes are
always an absolute `\x1b[r;cH`. Styles are written as differences, so
stopping an underline is `\x1b[24m` rather than a reset and a repaint. A run
of plain blanks to the end of a row becomes `\x1b[K`.

Colour depth is a value you pass in — `Ansi16`, `Ansi256`, `TrueColour` —
and a colour too rich for the depth is converted to the nearest entry of the
smaller palette. Detection belongs to whatever a program built on this does
with `$TERM` and terminfo; `lib/term.m31` deliberately leaves it out too,
for the same reason (its own header, "What is left out, and where it
belongs").

---

## The styling layer

`Theme` (`TUI_style.m31`) is a plain struct of `Style` fields, one per role —
`normal`, `dim`, `title`, `accent`, `highlight`, `selected`, `error`,
`added`, `removed`, `border` — each with a sensible default, so `Theme()` is
a working plain theme exactly as `Style()` is a working plain style.
Re-skinning a program built on this is replacing one `Theme` value rather
than hunting down every `Style(...)` a widget was built with; there is no
registry and no lookup by name, because a struct field is what this
language's interfaces already give for free.

`Span` and `Line` (`TUI_widget.m31`) are ratatui's `Span`/`Line` *concept*
through this library's own idiom rather than its API: a `Span` is a run of
text in one `Style`, and a `Line` is a `List<Span>` with a `render` method —
a `Widget`, exactly like `Block` and `Paragraph`, composing with `TUI_geometry`
the same way, rather than a separate "text buffer" type of its own.

```c
TUI_widget.Line([TUI_widget.Span("staged: ", style: th.dim),
    TUI_widget.Span("3 files", style: th.accent)])
    .render(buf, area);
```

---

## The primitives `apps/git/design.md` names

Six library primitives the interactive git client's design is built around,
none of them git-specific:

  - **`TUI_outline.Outline`** — nested, collapsible sections. A node's
    children are rows only while every ancestor down to it is expanded;
    folding a section removes its whole subtree from the flattened list in
    one step. Every node carries an `id`, the caller's own key, because
    position is not a stable address once a fold changes every row number
    below it.
  - **`TUI_jump_list.JumpList`** — a flat, selectable index of an `Outline`'s
    current rows, synced two ways: `sync_from` rebuilds it from the
    outline's flattened rows, `follow` highlights the entry matching wherever
    the outline's cursor now is, and `did_activate` moves the outline's cursor
    to whatever is selected here. Matching is always by `id`, never by row
    number, which is what keeps the two in step regardless of what is
    currently folded.
  - **`TUI_menu.WhichKey`** and **`TUI_menu.Transient`** — a popup of
    `(key, label)` pairs, and its sibling: the same popup with togglable
    `Switch`es and a live-rendered command preview. Both are built on one
    private `Popup` (a titled, sized-and-centred box of text rows) rather
    than as two unrelated components.
  - **`TUI_diff_view.DiffView`** — a diff with a cursor addressable down to
    the hunk *and* the line: `at()` answers a `Row` naming a hunk's header
    or one particular line of one particular hunk, which is the granularity
    "stage whatever is under the cursor" needs.
  - **`TUI_footer`** — `split(area, height)` cuts a fixed band off the
    bottom of the screen for a `Footer` (an ordinary `Line`) to draw into
    every frame, regardless of what the main view currently is.
  - **`TUI_scroll.Viewport`** and **`TUI_scroll.Scrollbar`** — the scroll
    offset and clamping math `Outline` and `DiffView` both need, and the bar
    that shows it, pulled out once rather than written a third and fourth
    time. `ListView` and `Table` keep their own inline copy, untouched.

---

## The application loop

`TUI_app.Loop` is the one module that touches a real terminal: raw mode via
`lib/term.m31`'s `raw()`, a read with a timeout via its `Reader`, a frame
via this library's own diffing renderer, a flush via its `Writer`. It is
the loop `examples/keys.m31` hand-writes, pulled out so a program built on
this library does not write it twice.

```c
TUI_app.Loop().run(view, on_event);
```

`view` is an ordinary `TUI_widget.Widget` — no new drawing interface exists,
because `Widget` already is "draw yourself into a rectangle of a buffer",
which is the whole of what a frame needs. `on_event` is a `TUI_app.Handler`,
a one-method interface in the same idiom `Widget` and every callback in this
language already use: `Action handle(term.Event ev)`, answering `Continue`,
`Redraw` or `Quit`. The loop itself decides *when* to redraw — after a
`Redraw`, or when `term.size()` says the window changed, since there is no
resize event — and whether that redraw is a full repaint or a diff against
the previous frame.

`examples/browse.m31` is a runnable proof: a small interactive file browser
built ONLY from widgets that existed before this loop (`ListView`,
`Block`, `Paragraph`) — j/k or the arrows move, Enter descends into a
directory, u or Backspace goes back up, q or Escape quits, and the terminal
is always restored on the way out, trap included, because that is what
`term.raw()`'s `Session` is for.

    M31_ROOT=/path/to/m31 bash scripts/build.sh examples/browse.m31 -o /tmp/browse && /tmp/browse

### Timer, mouse and paste (opt-in)

Everything below is off by default, so a `Loop()` with no arguments behaves
exactly as before.

```c
loop = TUI_app.Loop(animate_ms: 80, should_report_mouse: true,
                    should_report_hover: true, should_report_paste: true,
                    router: router);
loop.run_with(view, on_event, on_tick, on_mouse);
```

  - **Timer.** `animate_ms > 0` calls `on_tick.tick(elapsed_ms)` about that
    often; the read's timeout is the time to the next tick, so an idle program
    sleeps in the kernel and a stream of input cannot starve the timer.
    There is no monotonic clock in m31 (`docs/FRICTION.md` §22), so elapsed
    time comes from the wall clock, is clamped to one second, and a clock
    that goes backwards counts as zero.
  - **Mouse.** `should_report_mouse` turns reporting on at entry and off on
    exit; `should_report_hover` adds motion with no button down. Each report
    goes to `router`, which makes gestures (`Press`, `Drag`, `Release`, `Click` with a count of 1, 2
    or 3, `Hover`, `Leave`, and four `Scroll` directions) and
    finds the region of the last frame under the pointer. A drag is captured
    by the region it began in until its Release. With mouse reporting on,
    mouse events reach `on_mouse`, not `on_event`.
  - **Paste.** `should_report_paste` makes a paste one `Event.Paste`, which
    `TextInput` and `TextArea` insert as a unit (one undo step).

A trap, `os.exit` or a signal restores the terminal's raw mode but not these
modes (`docs/FRICTION.md` §23), so a program killed that way leaves the mouse
on until the shell is reset. `loop.leave_sequence()` is the exact bytes a
normal exit writes, for a program that wants to write them itself.

`examples/widgets.m31` uses all of it: a tab bar, a chart with a hover
readout, an outline and a diff view that take clicks and the wheel, a text
field, a gauge, a sparkline and a spinner, with the mouse and the keyboard
both working.

---

## Performance

A 200×50 frame — 10 000 cells, a full-screen terminal — built with
`gcc -O2`, mean of 200 runs:

| | time | bytes |
|---|---|---|
| paint a screenful of widgets | 1.60 ms | |
| `full` (every cell) | 0.62 ms | 12 533 |
| `diff` after one cell changed | 0.46 ms | 21 |
| `diff` with nothing changed | 0.47 ms | 0 |
| **paint + diff, the per-keystroke cost** | **2.10 ms** | |

The 60-frames-a-second budget is 16.67 ms, so a complete frame is about
**8× inside it**, and a frame in which nothing much moved is 21 bytes on the
wire. Run `M31_ROOT=/path/to/m31 bash tests/test.sh` for the numbers on your own
machine.

---

## Testing without a terminal

There are two test programs, `tests/tests.m31` (the stage 1 to 3 library:
175 `ok` lines plus rendered frames) and `tests/widgets.m31` (stage 4: 419
assertions plus rendered frames, golden renders of every new widget).
`tests/test.sh` runs each, compares it against its `.out` file (`tests/tests.out`,
`tests/widgets.out`; `--bless` rewrites them), checks for a `FAIL` line, checks
`__rc_live=0`, and checks that gcc and clang at `-O0` and `-O2` all agree.
Every widget and primitive added for stage 2 and stage 3 is in the first,
against synthetic buffers and synthetic `term.Event`s with no terminal
anywhere near it — `TUI_app.Loop.run` is the one function in this library
that cannot be (it opens raw mode and blocks on a real read), so its own
pure pieces are tested instead (`has_resized`, and a `Handler` written outside
the library dispatched directly, the same proof `Widget` gets below) and
the loop itself is proven by `examples/browse.m31` and `examples/widgets.m31`
against a real terminal. The stage 4 pieces that sit on the loop are pure:
the exact enter and leave byte sequences, the timer's timeout and clamping,
and the fold of routed answers are each a function with a test; mouse
handling is tested by feeding `term.Mouse` reports and a clock value to a
`Pointer` and a `Router` directly.

A buffer prints itself: `print(buf)` gives the rows as text, and
`buf.frame()` puts a rule around them so trailing spaces are visible and an
expectation is readable:

```
+----------------------------------------+
|The quick brown fox    dog, and keeps on|
|jumps over the lazy    going for quite a|
|dog, and keeps on      while after that.|
+----------------------------------------+
```

`TUI_diff.show(bytes)` does the same for escape sequences:
`\e[1;1H\e[1;4mab\e[22mcd\e[0m`.

Covered: a wide character at the right edge, both halves of a wide pair
overwritten, an emoji with a modifier, combining marks with and without a
base, an empty rectangle, a rectangle smaller than its border, a list longer
than its view, a table whose columns do not fit, every border style, all
three colour depths, and a widget written outside the library in each of the
three forms a one-method interface accepts.

---

## Left deliberately undone

Stage 4 built the whole of stage 2's original wishlist (`Gauge`, `Sparkline`,
`BarChart`, `Chart`, `Tabs`, mouse events, `TextInput`, `Spinner`), plus
`TextArea`. What still is not here, and why:

  - **A separate `Frame`/`Terminal` abstraction over the double buffer** —
    `TUI_app.Loop` owns the previous/next buffer pair and the decision between
    `TUI_diff.full` and `TUI_diff.diff` directly, which is the whole of what
    that abstraction would have been; a second type wrapping the same two
    fields would have no job left to do. This still stands.
  - **Restoring the mouse and the alternate screen on a trap, `os.exit` or a
    signal.** `lib/term.m31` restores termios on those paths and nothing else,
    because the safety story is a destructor and `on_exit` is not run
    (`docs/FRICTION.md` §23). A normal exit, including `Quit`, restores
    everything; a killed program leaves mouse reporting on. It needs a
    hook in `term`, not a workaround here.
  - **A monotonic clock.** The timer runs on the wall clock and is clamped
    (`docs/FRICTION.md` §22). It cannot be exact across a clock step.
  - **Editor-grade `TextArea`.** It has a cursor, a selection, word motion,
    undo and redo, wrapping, line numbers, paste and the mouse. It does not
    have rectangular selection, multiple cursors, folding, syntax colour, search, or input
    methods (IME composition); those belong to an editor built on it, not to
    a widget. Its model (lines of grapheme clusters, a `Position`) is
    documented in `TUI_textarea.m31`'s header for that purpose.
  - **A test of the real terminal path.** The loop's pure pieces are tested
    and every example was smoke-tested in a pty by hand, but there is no pty
    harness in `tests/test.sh`, so mouse mode on a real terminal is not a CI
    assertion, and CI runs only what the macOS and Linux bash and C
    compilers can run with no terminal.
  - **Chart extras.** No log axes, no secondary axis, no area fill, no
    annotations. Axis ticks are "nice numbers"; labels are fixed-point or
    short-form, because m31 has neither `str.replace` nor `math.sin`
    (`docs/FRICTION.md` §24).

`Tabs` deserves a plain account, because the earlier text of this section gave
a reason that was not the real one. It said a tabs widget "would be built for
a shape this library is not taking" — the locked design in `apps/git/design.md`
being one continuous, collapsible document. That was true of the one client,
and it was used as a reason to skip a widget that every general-purpose UI
library has, which is a reason about the client and not about the library. It
was left out because nothing in that client needed it. It is built now for
generality: a bar that scrolls when it overflows, selection by key and by
click, and closing as a request (`CloseRequested`) the program answers,
because only the program knows whether a tab has unsaved state.

`lib/term.m31` itself is not on this list: it already existed, complete and
tested, before this work started — see the module table above, and do not
believe an older copy of this README that says otherwise.

---

## Syntax highlighting

`TUI_syntax` (in `core/`) is a small lexical highlighter, not a parser: a
language is picked from a file name, every line is classified byte by byte
from the state the line before it left behind, and the classes are turned into
`TUI_style` styles in the terminal's own 16 colours (so they follow the user's
scheme and are right at every `Depth`). The tokenizer and the language
definitions (about eighty languages) are the shared
[`syntax`](https://github.com/qrazil/syntax) library, a dependency of this one
(pinned in `deps`); `TUI_syntax` is the adapter and the styling.

```c
str lang = TUI_syntax.language_of("src/main.rs", "");   // "rust"; "" when unknown
TUI_syntax.Highlighter hl = TUI_syntax.Highlighter(lang);
for (str text in lines) {
    hl.add(text);                                        // feed the file in order
}
bool dark = TUI_syntax.is_dark(os.env("COLORFGBG").or(""));
TUI_syntax.draw(buf, x, y, width, lines[i], hl.classes(i), TUI_style.Style(), dark);
```

`Highlighter.classes(i)` classifies lazily, through line `i`, and keeps the
result, so a long file costs one `classify` per line the first time it is
drawn and nothing when it is scrolled. `draw` clips to `width` and ends a
clipped line in the ellipsis; its `base` style is what plain text is drawn in
(an added line's green, say) and `over` is laid on every span (a cursor row's
reverse video). `TUI_diff_view.DiffView` takes the same thing as one field:
`view.language = "rust"` colours each hunk's code per token, classifying the
old and the new side of the hunk separately, keeping the `+`/`-` marker and
the plain text of added and removed lines in `add_style`/`remove_style`.
Passing no language is the previous behaviour exactly.

Classes: `NORMAL COMMENT STRING NUMBER KEYWORD TYPE FUNCTION PREPROC CONSTANT
OPERATOR SPECIAL TITLE`. Languages: c, cpp, go, javascript (ts, jsx, tsx),
json, m31, markdown, python, rust, sh, toml, yaml; a `#!` first line picks
python, sh or node for a file with no known extension.

---

## Layout

    deps                 project file: name, version, no dependencies
    core/                the buffer, geometry, text, style, clip, scale, mouse and widget primitives
    widgets/             the fourteen ready-made widgets built on core
    app/                 the diffing renderer (TUI_diff) and the application loop (TUI_app)
    examples/            demo, browse, widgets, bench: programs, built by scripts/build.sh
    tests/               test.sh (the one entry point), tests.m31, widgets.m31, syntax.m31 and their .out files
    scripts/             build.sh
    docs/                FRICTION.md, the notes from building this on m31

Folders are for finding files. Module names stay globally unique and keep the
`TUI_` prefix; `core/` never imports `widgets/` or `app/`, and `widgets/` never
imports `app/`.

## Build, test, release

Build and run a program: `M31_ROOT=/path/to/m31 bash scripts/build.sh examples/demo.m31 -o /tmp/d`.

Test: `M31_ROOT=/path/to/m31 LANGC=/path/to/m31c bash tests/test.sh` checks
formatting, compiles every test and example under gcc and clang at `-O0` and
`-O2`, compares each test's output to its `.out` file, checks `__rc_live=0`,
then runs the benchmark. CI runs it on Linux x86-64, Linux aarch64 and macOS
arm64 against the m31 release named by `M31_REF` in `.github/workflows/test.yml`.

Release: pushing a `v*` tag runs the same tests as a precondition and publishes
`tui-source.tar.gz` (the three folders, `deps`, README, `RENAMES.md`, scripts
and docs).
