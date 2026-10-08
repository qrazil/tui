# tui — a terminal user interface library

A buffer of grapheme clusters, a diffing renderer, ratatui's constraint
layout, width-aware text, a styling layer, thirteen widgets, and the
application loop that joins all of it to a real terminal. Nothing below the
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
renderer. See "Left deliberately undone" below for what stage 2's original
wishlist dropped, and why.

    M31_ROOT=/path/to/m31 bash scripts/check.sh                              # tests, then the benchmark
    M31_ROOT=/path/to/m31 bash scripts/build.sh examples/demo.m31 -o /tmp/d    # a static screen; run /tmp/d
    M31_ROOT=/path/to/m31 bash scripts/build.sh examples/browse.m31 -o /tmp/b  # an interactive one; run /tmp/b

**This library is not part of the standard library.** The compiler knows
nothing about it, nothing was added to `src/stdlib.rs`, and nothing was put
in `lib/`. It is an ordinary library: with m31 v0.2.0 or later a program
depends on it from its own `deps` file and imports each module by path.

    name myapp
    version 0.1.0
    tui https://github.com/qrazil/tui <ref>

    import tui.TUI_app;

(`tui path ../tui` uses a checkout in place.) The modules stay at the root of
this repository so those paths are `tui.<module>`; this repository's own
`deps` file is what marks the root, so `examples/` and `tests/` import the
modules by their bare names. What the language used to force instead is
written up in `docs/FRICTION.md` §1.

---

## The modules

Fourteen, in dependency order. They are all prefixed `tui` because module
names are globally unique and every unprefixed name a UI library wants is
either taken by `lib/` or likely to collide with a user's own file — see
`docs/FRICTION.md` §1. `TUI_app` is the one exception worth naming: it is the only
module that imports `lib/term.m31` and touches raw mode, a real read and a
real write; every other module below it is still pure computation against a
buffer.

| module | what is in it |
|---|---|
| `TUI_style` | `Colour`, `Style`, `Depth`, the attribute bits, the SGR escapes, and `Theme` |
| `TUI_geometry`  | `Rect`, `Constraint`, `Layout`, and the constraint solver |
| `TUI_text`  | `Align`, `wrap`, `truncate`, `pad`, `fit` — all in terminal columns |
| `TUI_buffer`   | `Cell` and `Buffer` |
| `TUI_diff`  | `full`, `diff`, and the small control sequences |
| `TUI_widget`| the `Widget` interface, `Block`, `Paragraph`, `Span` and `Line` |
| `TUI_list_table` | `ListView` and `Table` |
| `TUI_scroll`| `Viewport` (a scroll offset and its clamping/reveal math) and `Scrollbar` |
| `TUI_outline`| `Node`, `Outline` — the collapsible primary-view document |
| `TUI_jump_list`  | `Entry`, `JumpList` — a flat index synced two ways to an `Outline` |
| `TUI_menu`  | `Popup`, `WhichKey`, `Switch`, `Transient` — one base, two overlays |
| `TUI_diff_view`| `Kind`, `Line`, `Hunk`, `DiffView` — a diff with a cursor addressable to the line |
| `TUI_footer`| `split`, `Footer` — a screen band reserved regardless of the main view |
| `TUI_app`   | `Action`, `Handler`, `Loop` — the application loop, over `lib/term.m31` |

Each file's header comment is its reference documentation.

---

## The shape of a program

```c
import TUI_buffer;
import TUI_diff;
import TUI_geometry;
import TUI_list_table;
import TUI_style;
import TUI_text;
import TUI_widget;

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
wire. Run `M31_ROOT=/path/to/m31 bash scripts/check.sh` for the numbers on your own
machine.

---

## Testing without a terminal

`tests/tests.m31` prints a report — 163 assertions plus rendered frames — and
`check.sh` compares it against `tests/tests.out`, checks for a `FAIL` line, checks
`__rc_live=0`, and checks that gcc and clang at `-O0` and `-O2` all agree.
Every widget and primitive added for stage 2 and stage 3 is in there,
against synthetic buffers and synthetic `term.Event`s with no terminal
anywhere near it — `TUI_app.Loop.run` is the one function in this library
that cannot be (it opens raw mode and blocks on a real read), so its own
pure pieces are tested instead (`has_resized`, and a `Handler` written outside
the library dispatched directly, the same proof `Widget` gets below) and
the loop itself is proven by `examples/browse.m31` against a real terminal.

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

Stage 2's original wishlist named a few things not built here, each a
named, separate omission rather than a silent one:

  - **`Gauge`, `Sparkline`, `BarChart`, `Chart`** — dashboard widgets. Nothing
    in a text-document-shaped client (`apps/git/design.md`'s own frame, and
    the only concrete client this library is being built toward so far)
    has a role for one, and adding a chart widget on spec, with nothing to
    render it TO, is exactly the guessing this library's design has avoided
    elsewhere.
  - **`Tabs`** — the locked design is one continuous, collapsible document
    with a jump list beside it, not a set of screens to switch between; a
    tabs widget would be built for a shape this library is not taking.
  - **Mouse events** — the locked design is keyboard-driven throughout.
    `lib/term.m31` already decodes SGR mouse reports (`Event.Mouse`) for
    whatever does want them; there is simply nothing here that reads one.
  - **`TextInput`** — no named client need yet. Nothing in
    `apps/git/design.md` asks for free-text entry (a commit message is the
    likely first caller, and it is not designed yet); adding it now would be
    guessing at a shape.
  - **`Spinner`** — lowest priority on the original list, and nothing asked
    for it in the course of building the six primitives above, so it did
    not fall out for free and was not built speculatively.
  - **A separate `Frame`/`Terminal` abstraction over the double buffer** —
    `TUI_app.Loop` ended up owning the previous/next buffer pair and the
    decision between `TUI_diff.full` and `TUI_diff.diff` directly, which is
    the whole of what that abstraction would have been; a second type
    wrapping the same two fields would have had no job left to do.

`lib/term.m31` itself is not on this list: it already existed, complete and
tested, before this work started — see the module table above, and do not
believe an older copy of this README that says otherwise.
