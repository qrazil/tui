# Writing a terminal UI framework in this language

Stage 1 of three: cells, a buffer, a diffing renderer, a constraint layout,
width-aware text and five widgets — 2 766 lines of library, 550 lines of
tests, no terminal touched. What follows is everything that got in the
way, everything that was missing, and the things that were genuinely better
than the languages this is measured against.

Ordered by how much it cost, not by how annoying it was.

> **Three of the five changes §21 asks for have since been made**, and this
> file has not been rewritten to hide that it asked for them — the source has
> been changed to use them, so the diff against this log is the evidence.
> The range `for` of §4 (§21.3), so `Buffer.fill` — the loop this report hung
> in — no longer has an increment for a `continue` to lose; a `case` arm that
> may omit its bindings (§6, §21.5), which takes `TUI_geometry.Constraint`'s four
> methods down by a third; and a formatter that keeps the author's
> parentheses, so §18's `(off & BOLD) != 0` stays as written. A character
> literal is in as well. §1, §2 and the standard library asks of §21.4 are
> untouched. `docs/reference.md` §1.5, §5.5, §5.6 and §6.1 have the rules.

---

## 1. There is no way to depend on a library

This is the largest single problem and it is not a small feature request.

**A module is a file, and `import` resolves against the directory of the
entry file** (`src/modules.rs`: `entry_path.parent()`). So a program that
uses this library has to be *in this directory*. There is no include path,
no manifest, no `import tui/geom`, no way to say where a dependency lives.
The three ways a user could get at this library today are: copy its seven
files into their own directory, put their program in `apps/tui/`, or
symlink. All three are wrong.

**Module names are globally unique across a program**, because the name is
the basename. That is the second half of the problem, and it is what shaped
the whole public API:

| what the file should be called | what it is called | why |
|---|---|---|
| `style.m31` | `TUI_style.m31` | might collide with a user's own |
| `geom.m31` / `layout.m31` | `TUI_geometry.m31` | same |
| `buffer.m31` | `TUI_buffer.m31` | same |
| `text.m31` | `TUI_text.m31` | **`text` is a standard library module** |
| `diff.m31` / `render.m31` | `TUI_diff.m31` | might collide |
| `widget.m31` | `TUI_widget.m31` | same |
| `list.m31` | `TUI_list_table.m31` | same |

Sixteen names are already taken by `lib/` (`args base64 csv date fs html
http io json math net os random sort text unicode`), and every one of them
is a word a UI library wants. So every call site in every program that uses
this reads

```c
TUI_geometry.Layout(TUI_geometry.Direction.Vertical, [TUI_geometry.Constraint.Length(3),
    TUI_geometry.Constraint.Fill(1)], margin_y: 1)
```

where it should read

```c
layout.Layout(layout.Direction.Vertical, [layout.Constraint.Length(3),
    layout.Constraint.Fill(1)], margin_y: 1)
```

The `tui` prefix is not decoration; it is a manual namespace, hand-applied,
because the language has no other. It is Objective-C's `NS`, and it is here
for exactly the reason it was there.

**There is no re-export.** A library normally has one front door — one
module a user imports that brings the rest with it. That cannot be written:
a module cannot re-export another module's names, there are no type aliases
(§3.6), and a forwarding function can forward a function but not a *type*.
So a program must write all seven imports:

```c
import TUI_buffer;
import TUI_diff;
import TUI_geometry;
import TUI_list_table;
import TUI_style;
import TUI_text;
import TUI_widget;
```

and know which of the seven each name lives in.

**What I would want, in order of preference.**

1. **A path is the module name.** `import tui/geom;` reads
   `<root>/tui/geom.m31` and binds it as `geom` locally (or `tui.geom` if
   the short name is taken). Roots: the entry file's directory, then
   anything named on the command line or in a one-line manifest. This makes
   the name *hierarchical*, which is the actual fix — `tui/text` and the
   standard `text` stop being the same name, and the `tui` prefix
   evaporates. OCaml pays the flat-namespace cost this language currently
   pays and it is widely regretted there.
2. **A namespace per dependency.** If a manifest names dependencies, then
   `tui.geom.Rect` where `tui` is the dependency and `geom` its module.
   Collisions become structurally impossible rather than a convention.
3. **`pub import`** — one line in a `tui.m31` that re-exports the seven, so a
   library has a front door. Additive; it does not change what any existing
   program means.

I do *not* want a package manager, a lockfile or a registry. A search path
and a hierarchical name would be enough, and both are additive.

The one thing that is genuinely good about the current arrangement: the
standard library is embedded in the compiler, so `import unicode` works with
no configuration on any machine. Whatever replaces this should keep that
property for `lib/` and only extend it outwards.

---

## 2. A module name shadows a field in a file that never imported it

**Fixed.** A module's name is now in scope only in the file that imported it
(reference §2.1), so `Paragraph.text` is the field again and the workaround
below is gone from `TUI_widget.m31`. A file that imports `text` *and* declares
a field called `text` is refused at the field, naming both (§4.3).

This is a bug, and it is the only thing here that produced a diagnostic I
could not act on.

`TUI_widget.m31` declares

```c
pub type Paragraph {
    pub str text;
    ...
}
```

and a method on it did

```c
for (str l in text.split("\n")) { ... }
```

That compiled, and kept compiling, through every test in this directory.
Then `bench.m31` — a *different* file — added `import date;`, and
`lib/date.m31` imports `text`, and the build broke:

```
apps/tui/tuiwidget.m31:253:23: `text` has no function `split`
    |
253 |     for (str l in text.split("\n")) {
    |                       ^
```

`TUI_widget.m31` does not import `text`. It never did. A module pulled in
*transitively by another file of the same program* took the name away from
my field. Reference §4.1 says nothing shadows "a module the file imports" —
this is a module the file does not import, and the rule as implemented is
"any module anywhere in the program".

That makes every one of these unusable as a field name in a library, and
unusable *non-locally*, so the breakage arrives when a downstream program
adds an unrelated import: `args base64 csv date fs html http io json math
net os random sort text unicode`. `text`, `args`, `date`, `json`, `net`,
`math` are all names a struct wants.

**The workaround** (it is in the source, with a comment):

```c
str whole = text;                       // bind the field to a local first
List<str> out = whole.split("\n");
```

There is no other way out: `this.text.split(..)` is refused by §4.3, which
exists to stop the same read being written two ways — but here it is the
*only* unambiguous spelling and it is banned.

**What I would want:** a bare name resolves to a field of the receiver
before it resolves to a module, and a module name is in scope only in the
file that imported it. Failing that, the diagnostic should say so: "`text`
is the standard library module here; `Paragraph.text` is a field of the
receiver — bind it to a local, or rename the field."

---

## 3. `List` is a predeclared name, so the list widget is `ListView`

Every TUI library has a widget called `List`. This one cannot: nothing may
shadow a predeclared type name (§4.1). `TUI_list_table.ListView` it is. Small, but
it is the first name a user looks for and it is not there.

---

## 4. Loops: no `+=`, no range `for`, and `continue` skips the increment

Every loop in this library is

```c
int i = 0;
while (i < n) {
    ...
    i = i + 1;
}
```

There are 64 of these `i = i + 1;` lines, one for each of the 64 counted
loops in the library and its tests (48 in the library alone). That is tolerable on its
own. What is not tolerable is that a `continue` in the body skips the
increment, so the renderer is full of

```c
if (w == 0) {
    x = x + 1;
    continue;          // must repeat the increment, or hang
}
```

I wrote an infinite loop this way once, in `Buffer.fill`, and found it only
because the test run stopped producing output. In a three-clause `for` the
increment belongs to the loop and `continue` cannot lose it; in a range
`for` there is no increment to lose.

```c
for (int i in 0 .. n) { ... }         // what I wanted
```

This is the single change that would most improve the *body* of this code.
It is additive, it needs no new type (a range form can desugar to the same
while), and it removes an entire class of bug that the language otherwise
leaves entirely to the programmer — which is odd in a language that traps on
integer overflow because it does not trust the programmer with that.

`for (T x in xs)` is good and I used it wherever the index was not needed.

---

## 5. No ternary, so a two-way choice is four lines

`Block.inner` has four of these in a row:

```c
int l = 0;
if (is_drawn(LEFT)) {
    l = 1;
}
int r = 0;
if (is_drawn(RIGHT)) {
    r = 1;
}
```

twenty lines for what is

```c
int l = is_drawn(LEFT) ? 1 : 0;
```

or, better and in keeping with the language's own taste, an `if` that is an
expression. I am not asking for C's `?:`; I am pointing out that "choose one
of two values" happens constantly in layout code and currently costs a
mutable local and a statement.

---

## 6. Exhaustive `match` with no `default` is 20 lines to say one thing

`Constraint` has six variants. `Constraint.weight()` means "the weight of a
`Fill`, and nothing for the rest":

```c
int Constraint.weight() {
    match (this) {
        case Length(int v): { return 0; }
        case Percentage(int v): { return 0; }
        case Ratio(int a, int b): { return 0; }
        case Min(int v): { return 0; }
        case Max(int v): { return 0; }
        case Fill(int v): { ...the real answer... }
    }
}
```

`TUI_geometry.m31` has four such methods — `kind`, `weight`, `preferred`,
`to_str` — 28 `case` arms between them, and about 110 of the file's 605
lines. The formatted source is three
lines per arm, so it is worse than it looks above.

I understand and agree with the argument in §5.6: a `default` means adding a
variant silently does the wrong thing somewhere. But that argument is about
enums whose variants will grow, and it is a *whole-program* argument being
applied to a `match` in the same module as the `enum`, where adding a
variant means editing this file anyway. Something narrower would pay for
itself: `case _:` allowed only when the enum is declared in the same module,
or an `else` arm that the compiler warns about (there are no warnings) — or,
best, bindings that may be omitted, so a payload-ignoring arm is
`case Length:` rather than `case Length(int v):` with an unused `v`.

The smallest version of this I would take: **let an arm omit its bindings.**
That alone halves the noise, because most of these arms ignore the payload.

---

## 7. Comparing two enums is quadratic in source

My first `Colour.eq` was a `match` inside a `match`: nine arms, forty lines,
to say "are these the same colour". The fix was to project both to an `int`:

```c
pub bool Colour.eq(Colour o) {
    return code() == o.code();
}
```

which is better code and I kept it. But the projection existed because I
needed a packed representation anyway; without that excuse the nested match
is the only way, and it does not scale past three variants. A derived
structural `eq` for enums — the one thing every language with sum types has
— would be worth more here than in most languages, because §3.9 makes `eq`
a requirement for map keys.

---

## 8. `match` on a two-variant enum needs empty blocks

```c
int along = box.w;
match (dir) {
    case Horizontal: {
    }
    case Vertical: {
        along = box.h;
    }
}
```

An empty block to say "nothing changes in this case". `Layout.split` does
this three times. `Direction` should be an enum — it is exactly the type the
language is for — but reading one bit out of it costs six lines.

---

## 9. Appending a `str` to a `bytes` allocates — and it was measurable

This is the one missing standard library method with a number attached.

The renderer emits one `bytes` per frame. Writing a cell's symbol into it is

```c
out.extend(s.to_bytes());          // what I wrote first
```

and `to_bytes()` **allocates a whole `bytes` object per cell** (§6.5: "a
`bytes` copy of the same octets"), which is then copied and thrown away.
Ten thousand cells, ten thousand allocations, per frame.

What I wanted:

```c
out.extend_str(s);                 // or out.push_str(s)
```

What I had to write, inside `TUI_buffer` where the `str` is in hand without a
retain:

```c
pub void Buffer.append_symbol(bytes out, int x, int y) {
    str s = symbols[at(x, y)];
    int n = s.size();
    int i = 0;
    while (i < n) {
        out.push(s.byte_at(i));
        i = i + 1;
    }
}
```

**Measured, 200×50 full redraw: 1.24 ms → 0.77 ms.** A byte-at-a-time loop
through two runtime calls beat the library method by 40 %, which means the
library method should not exist in that shape. `bytes` has `extend(bytes)`
and `str` has `to_bytes()`; the pair that is missing is the one every
program that builds a byte string from text needs.

---

## 10. A returned `str` costs a retain and a release, and it dominated the diff

The renderer's inner question is "is this cell the same as last frame's".
The obvious spelling, across module boundaries, is three accessor calls:

```c
bool has_changed(Buffer prev, Buffer next, int x, int y) {
    if (prev.width_at(x, y) != next.width_at(x, y)) { return true; }
    if (prev.style_at(x, y) != next.style_at(x, y)) { return true; }
    return prev.symbol_at(x, y) != next.symbol_at(x, y);      // two owned strs
}
```

Returns are owned, +1 (§7.2), so each `symbol_at` is a retain in the callee and
a release in the caller, and `rc_inc`/`rc_dec` are deliberately out-of-line
(`gates.sh` enforces that they stay in `rt.c`). Four non-inlinable calls per
cell, forty thousand per frame.

The fix was to put the comparison where the data is:

```c
pub bool Buffer.is_same_cell(Buffer other, int x, int y) { ... }
```

**Measured, 200×50 diff with nothing changed: 1.36 ms → 0.50 ms.** A 2.7×
speed-up from removing refcount traffic alone.

I am not complaining about refcounting — it is the right trade and the
destruction guarantees are worth it. Two observations: (a) a caller cannot
see this cost, because the signature says `str` and nothing says "this is
two atomic-free but out-of-line calls"; (b) the obvious API shape and the
fast API shape differ, and the language pushes you to the obvious one. A
`str` that is *borrowed* for the duration of a call — the thing `this` and
parameters already are — would remove the cost without changing the
signature.

---

## 11. `unicode.width` has no ASCII fast path, and it cost half the frame

`unicode.width` and `unicode.graphemes` run UAX #29: a state machine per
code point, a binary search per cluster. Correct, and not cheap. A terminal
UI asks the question thousands of times a frame, almost always of text that
is entirely printable ASCII, where the answer is `s.size()`.

I could not change `lib/`, so `TUI_text` has its own:

```c
bool is_plain(str s) {                     // every byte printable ASCII?
    int n = s.size();
    int i = 0;
    while (i < n) {
        int b = s.byte_at(i);
        if (b < 32 || b > 126) { return false; }
        i = i + 1;
    }
    return true;
}

pub int width(str s) {
    if (is_plain(s)) { return s.size(); }
    return unicode.width(s);
}
```

**Measured, painting a 200×50 screen: 3.62 ms → 2.04 ms.** The same scan
gives `clusters()` a fast path (one byte per cluster in that range), which
is the other half of the win.

This belongs in `lib/unicode.m31`, is provably equivalent inside
0x20..0x7E — one byte is one code point, one code point is one cluster, one
cluster is one column — and is about eight lines. It is the single highest
value change available to the standard library for this kind of program. As
it stands, every program that measures text will write this function again,
and some of them will get the boundary wrong.

A second thing missing from `unicode`: **a way to walk grapheme clusters
without allocating one `str` per cluster.** `graphemes(s)` builds
`List<str>`, so wrapping a 1 400-character paragraph allocates 1 400
strings. The private `cluster_starts` in `lib/unicode.m31` already returns
the offsets; making it `pub` would let a wrapper walk the text with
`substr` only where it actually cuts.

---

## 12. What is missing from `str` and `bytes`

  - **`s.index_of(sub, from: i)`.** `lib/text.m31` explains at length why it
    cannot provide this without being quadratic. It is right, and the
    built-in is still missing.
  - **`extend_str` / `push_str` on `bytes`** — see §9.
  - **`saturating_add` / `saturating_mul`.** `wrapping_*` exist for hashes.
    A layout computes `amount * weight` where both come from a caller;
    overflow traps, so I clamped the `Fill` weight to 1 000 000 by hand.
    Saturation is the right answer for a size, and wrapping is the wrong one.
  - **`clamp`**. `lib/math` has `min` and `max`; hand-written three-line
    clamps appear a dozen times here. I did not import `math` in the hot
    modules to keep the dependency list short, which is itself a symptom
    of §1.

---

## 13. Module constants cannot hold a user type

§4.5: a constant's type is a scalar, or an `Array`/`List`/`Map` of scalars.
So the five border character sets, which are obviously

```c
type BorderSet { str h; str v; str tl; str tr; str bl; str br; }
const Array<BorderSet> BORDER = [ ... ];
```

are instead a flat `Array<str>` of 24 entries read as `BORDER[row * 6 + 2]`.
It works, it is commented, and it is an index computation standing where a
field name belongs. The same applies to the xterm palette and to anything
else a UI library wants as a static table — which is most of what a UI
library wants as a static table.

---

## 14. No mutable module state, which is mostly fine and occasionally not

The test harness needs a pass/fail counter. There is no module-level
variable, so:

```c
type Tally { int pass = 0; int fail = 0; }
void check(Tally t, str name, str got, str want) { ... }
```

and `t` is threaded through all 110 checks and all eleven test functions.
That is a small, honest cost and the reasoning in
`docs/module-state-decision.md` is sound. Worth recording only because a
test harness is the first thing anyone writes and it is the first thing that
meets this.

---

## 15. No way to test that something traps

`Buffer.get` out of bounds traps. `diff` on mismatched rectangles traps.
`Colour.Indexed(300)` traps. All three are documented and **none is tested**,
because a trap aborts and there is no `catch` (correctly — §9). The corpus
has `corpus/traps/` for exactly this, but that machinery belongs to the
compiler's own test corpus and this library is deliberately not in it.

What would fix it for libraries: nothing in the language. A convention would
do — a `traps/` subdirectory beside a library's `check.sh`, one file per
expected trap, with the expected message. I did not build one because it
would be a fifth ad-hoc test runner in this repository; it is worth deciding
once, centrally.

---

## 16. Sentinels where an `Option` belongs

`ListView.selected` is `-1` for "nothing selected" and
`highlight_symbol` is `""` for "none". Both are exactly what `Option` exists
to prevent, and I used them anyway, because the alternative is

```c
pub Option<int> selected = Option<int>.None;
```

and then every read inside `render` is a four-line `match` — in a method
that runs per frame, and reads it once per row. `o.or(-1)` gives the
sentinel back in one call, which means the honest version is *strictly
more* typing for the same value.

This is a real trade rather than a defect, and the language is on the right
side of it in general. It is recorded because a library's public fields are
where sentinels become other people's problem, and `Option` did not win
here.

---

## 17. Diagnostics: mostly very good, two notes

Good ones, quoted exactly as they arrived:

```
apps/tui/bench.m31:84:51: expected a name, found `bytes`
   |
84 | void report(str name, int total_ns, int runs, int bytes) {
   |                                                   ^
```

Right position, right token. `bytes` is a type keyword and I used it as a
parameter name; the message could add "`bytes` is a type keyword" but the
caret says everything.

The bad one is §2's: `` `text` has no function `split` `` tells you what the
compiler decided without telling you there was a decision.

One more: the shadowing rule catches a **parameter** that collides with a
**module-level function in the same file**. I had `pub int cols(str s)` and
`Paragraph.lines(int cols)` in `TUI_widget.m31`; the parameter is refused.
The message was clear, and the rule is right, but it is a rule that reaches
further than people expect — a library author has to keep every parameter
name clear of every function name in the module.

---

## 18. The formatter

It preserves meaning (checked against every test in this directory) and it
is idempotent. Two things it does that I would rather it did not:

  - **It removes blank lines between top-level statements.** `demo.m31` is a
    program made of top-level statements with `// the banner`, `// three
    panels`, `// the status bar` section comments; after `m31c fmt` every
    blank line between them is gone and each comment is glued to the
    statement above it. The file is materially harder to read after
    formatting than before.
  - **It strips the parentheses in `(off & BOLD) != 0`**, giving
    `off & BOLD != 0`. That is correct under §6.1's Python precedence, and I
    think the precedence choice is right — but the parentheses were there
    for the reader who learned C, and the formatter took away the author's
    ability to leave them.

It also glued a run of `const` declarations together across a blank line I
had put there to separate the public attribute bits from the private
packing constants. Same class of thing.

---

## 19. Run-time surprises: there were none

This deserves its own heading because I expected otherwise.

  - **`__rc_live=0`** on the first run of the test program, and on every run
    since. No leak anywhere in 2 600 lines of library with lists of lists,
    lists of strings, enums carrying payloads, lambdas, interface values and
    structs mutating their own fields. I did not once think about memory.
  - **gcc and clang, `-O0` and `-O2`, all four builds agree byte for byte**
    on 500 lines of test output, including escape sequences and Unicode.
  - **The emitted C compiles with `-Wall -Wextra` and no warnings** for all
    three programs here.
  - **No trap I did not write myself.** No index error, no overflow, no
    `substr` landing inside a character — and this library does a great deal
    of byte arithmetic on UTF-8.

For a compiler this young, on a program this size, that is a much better
record than I assumed I was going to report.

---

## 20. What was genuinely good

**Field defaults, with the mandatory-positional / optional-named rule.** This
is the best thing in the language for library design and it is not close.
Every widget here is one construction:

```c
TUI_widget.Block(title: "Files", borders: TUI_widget.Borders.Rounded, pad_x: 1)
TUI_widget.Paragraph(body, align: TUI_text.Align.Center, scroll: 4)
TUI_list_table.ListView(names, selected: 3, highlight_symbol: "> ")
```

ratatui needs a builder — `Block::default().borders(Borders::ALL).title("x")`
— and every builder is a hand-written method per field, a `mut self` dance,
and a rule about which order you may call them in. Here there is no builder,
no order, and you cannot pass the arguments in the wrong slots because the
optional ones are named and the mandatory ones are not. `Style()` is the
default style and `Style(foreground: red)` is the one you meant. It made the whole
API smaller than its Rust equivalent by a large factor.

**Structural one-method interfaces.** The widget protocol is

```c
interface Widget { void render(TUI_buffer.Buffer b, TUI_geometry.Rect area); }
```

and a struct with fields, the bare name of a function, and a lambda written
at the call site all satisfy it, with nothing declared anywhere:

```c
draw(Bar(7, 10), b, area);                                  // a struct
draw(ruler, b, area);                                       // a function
draw((TUI_buffer.Buffer buf, TUI_geometry.Rect a) =>
     buf.set_str(a.x, a.y, "a lambda is a widget", st), b, area);
```

All three are in `tests.m31`. Nothing in ratatui (a trait and an `impl`
block) or bubbletea (an interface with three methods and a `tea.Msg` type
switch) is this light. A user's own widget is genuinely no harder to write
than a shipped one, which is the property the whole design was aiming at.

**Enums with exhaustive `match`** for `Constraint`, `Borders`, `Align`,
`Depth` and `Colour`. The complaint in §6 is about the syntax, not the
feature: a `Constraint` that carries its payload, cannot be constructed
wrongly and cannot be matched incompletely is exactly right for a layout
engine, and it is why the solver is 250 lines rather than a class hierarchy.

**`bytes` with the bit operators and no `char` type.** The renderer builds
escape sequences by pushing integers, packs a whole style into one `int`
with `|` and `>>`, and never once needed a cast, a `u8`, or a decision about
signedness. Python's bitwise precedence (§6.1) means `x & 1 == 0` does what
it looks like, which I relied on throughout and never checked.

**`unicode.graphemes` and `unicode.width` existing at all.** This library's
central promise — a cell is a grapheme cluster, a wide character is two
cells — is only writable because the standard library did the hard part
first. Most languages make you find a crate for this.

**The trap-on-overflow default.** In a layout engine every number is a size
and every wrong number is a silent misdraw. I would rather it stop.

**Destructors and the ownership rules never came up.** That is the
compliment: 2 766 lines and the memory model was invisible, except where I
went looking for speed (§10).

---

## 21. Summary: the five changes worth making

1. **A way to depend on a library that is not in `lib/`** — a search path and
   a hierarchical module name (§1). Everything else here is a paper cut;
   this one decides whether libraries exist.
2. **A bare name resolves to the receiver's field before a module** (§2).
   This is a bug, it breaks at a distance, and it has no clean workaround.
3. **A range `for`** (§4). Removes the most common shape of loop bug in this
   code, and 64 lines that say nothing.
4. **An ASCII fast path in `unicode.width`/`graphemes`, and `bytes.extend_str`**
   (§9, §11). Two small, measurable standard library changes worth 2× on a
   frame.
5. **Let a `case` arm omit its bindings** (§6). The cheapest possible
   improvement to the noisiest thing in the source.

---

# Stage 4: widgets, a timer and the mouse (v0.3.0)

Thirteen modules were added against m31 v0.3.2: the
dashboard widgets, `Spinner`, `TextInput`, `TextArea`, `Tabs`, a mouse router,
and a timer and mouse mode on `TUI_app.Loop`. Nothing in it needed a
workaround in another language, and nothing was a stop-and-ask. The sections
that follow are the things that cost something, in the same order as above.

## 22. There is no monotonic clock

An animation wants "milliseconds since the last tick", and the only clock
`date` offers is wall time (`date.now_ns`). A wall clock can step: an NTP
correction, a suspended laptop, a person changing the time. An animation
driven by it leaps forward or freezes.

What `TUI_app` does instead: it measures elapsed time from the wall clock,
**clamps** it to `MAX_ELAPSED_MS` (one second) so a forward step is one big
tick and not a leap, counts a backwards step as zero, and **resynchronises**
the next due time from now. That is correct enough for a spinner and a clock
and it is not what a timer should have to do. A monotonic `date.mono_ns()`
(or `time.monotonic`) would delete `clamp_elapsed` and its tests.

## 23. A trap, `os.exit` or a signal restores termios and nothing else

`term.raw()` hands back a `Session` whose destructor restores the terminal,
and the header of `lib/term.m31` is right that this covers a trap. What it
restores is the termios state. Stage 4 turns on three more things that need
undoing: the alternate screen, mouse reporting (`term.MOUSE_ON`) and bracketed paste
(`term.PASTE_ON`). `Loop.run_with` writes `leave_sequence()` on a normal
exit, a `Quit`, so those are fine. On a trap, `os.exit` or SIGINT/SIGTERM the
destructor runs only the termios part, so the shell is left with mouse
reporting on and every click prints `^[[<0;12;5M`.

The fix belongs in `term`, not here: `Session` needs an `on_exit` string (or
a registered cleanup) that the trap and signal paths also write. This library
cannot do it from m31, because there is no way to register code for a trap,
and writing it in Rust was never an option. It is documented in the README
and `Loop` writes the sequence on every path it controls.

## 24. Small standard library holes

  - **No `str.replace`.** Changing "1.5" to "1,5", or stripping a trailing
    zero, is a loop over clusters. `TUI_scale.format_fixed` and
    `format_short` build their strings by hand.
  - **No `math.sin`.** A chart demo wants a smooth curve; the example uses
    a small linear congruential generator for its load samples instead. A
    chart widget does not need `sin`, a demo does.
  - **`Option` has no `unwrap`** and no `or_else`. A `match` with `case Some`
    and `case None` is the only way in, which is the right default and
    verbose for "I just checked". This is a repeat of §6.
  - **`List.remove` is `remove_at(i)`.** A name, not a gap, but it cost a
    failed build because `remove` is what every other language calls the
    by-value version.
  - **A float literal `1e30` is not accepted.** `TUI_chart` starts its
    running minimum and maximum from `1000000000000000019884624838656.0`,
    which is 1e30 written out. A literal with an exponent would read better.

## 25. The lint's naming rules and natural API names

`m31c lint` requires boolean-returning names to start with `is_`, `has_`,
`can_`, `should_`, `did_`, `was_`, `needs_` or `will_`, and names to be at
least three characters. Both are good rules and both changed public API:
`Spinner.did_tick` and `did_advance`, `Tabs.did_select`, `Gesture.has_mods`,
`Scale.is_inside`. Most read fine. `did_` is the awkward one: it is
past-tense for a query ("did the spinner advance on this tick") and it
silently asks the caller to know whether the answer is about this call or
about the object's history. A convention in the docs that `did_` means "the
last call changed something" would remove the doubt, and this library
follows it: `Outcome.Ignored` means no state changed, and `did_*` is true
otherwise.

## 26. A match must name every variant, and the order is not free

An enum `match` has no `default:`, so adding `Hover` and `Leave` to
`TUI_mouse.Kind` made every `match` on it in the library fail to compile
until it was extended, and most of the extensions were "this does
nothing here". That is exactly what the rule is for (a widget that
ignores a hover now says so), and it is why none of the eleven is wrong. The
cost is that an additive change to an enum is not additive for callers:
`Kind` having ten variants is part of this library's API, and a program with
its own `match (gesture.kind)` breaks on the next release that adds one. The
`deps` bump is 0.2.0 to 0.3.0 for this reason as well as the new modules.

A `default` that is only allowed on `match` over an enum from another module
would give a library room to grow. This is a suggestion, not a request: the
exhaustiveness rule is the reason a widget that ignores a gesture says so.

## 27. What was good, again

**Structural one-method interfaces.** `Ticker`, `MouseHandler`, `Validator`
and `Widget` are all one method, so the demo passes lambdas that close over a
single state object (there is no mutable module state, §14, and it did not
matter). No registration, no base class.

**The golden test shape.** A widget renders into a `Buffer`, `frame()` rules
it, and the expectation is the picture. 419 assertions in the widget test
program, many of them pictures, and a wrong picture shows where.
`Router` and `Pointer` are the same: a report and a clock value in, gestures
out, and no terminal.

**`__rc_live=0` as a test result.** Gesture lists, snapshots for undo and
the router's regions are allocated and dropped all over, and the check that
nothing leaked is free.

**One real bug that the tests found, not review.** A drag ending in a Release
left the router's capture set, so the next click in another region went to
the old one. The fix was small (`Gesture.is_press_end`); it was found by a test that
drove the whole router through a sequence, which a test of one gesture at a
time would not have done.

## 28. 0.10.0: the terminal features (m31 v0.6.0)

Written against m31 v0.6.0 (`lib/term.m31`, `text.unicode` 16.0). Nothing here stopped the work; each item is
something `term` or `unicode` could say or do so a library does not have to.

**`term` has no vocabulary for what a modern terminal says.** The decoder reads `CSI u` (so kitty keys arrive
as `Key`) and nothing else new: a focus report (`CSI I`, `CSI O`), the answer to a mode query (`CSI ? 2026 ; 2 $ y`),
the kitty flags answer (`CSI ? 0 u`), the device attributes answer (`CSI ? 62 ; 22 c`) and a modifyOtherKeys report
(`CSI 27 ; 5 ; 105 ~`) are all `Event.Unknown(bytes)`, and `TUI_app` / `TUI_keys` decode each by hand
(`focus_report`, `sync_reply`, `reply_kind`, `modify_other_key`). An `Event.Focus(bool)` and an `Event.Reply` with the
private marker, the numbers and the final byte would make four of those five pure `match`es. The sequences
that turn the modes on and off (`1004`, `2026`, kitty push and pop, `modifyOtherKeys`, OSC 52, OSC 8) are strings in
this library for the same reason `MOUSE_ON` is one in `term`; some of them belong beside it.

**`Session.on_exit(str)` appends**, so what is written on the way out is fixed in order when raw mode is entered,
and a mode that is only known later (the kitty protocol, once the terminal has answered) cannot join it in the
right place: the kitty flags have a stack per screen, so the pop has to come before the alternate screen is left. `Loop`
writes the pop itself on the normal path (and that is the only path `on_exit` covers anyway, §23). A way to set the
whole leave string again would fix it.

**`text.unicode.width` against UTS #51** (checked with every fully-qualified sequence of emoji-test.txt 15.1, 3773
of them, and python-wcwidth as a second opinion; `tests/terminal.m31` pins the numbers):

- an emoji modifier after a text-presentation base (261D 1F3FB and 44 more) is one column; UTS #51 says an emoji modifier
  sequence has emoji presentation. `wcwidth` says one as well. `TUI_text.cluster_columns` says two.
- a variation selector alone (U+FE0F) is two columns, and U+3164, U+302E, U+302F and U+16FF0..1 as well (wcwidth: zero).
  `TUI_text` corrects only the selectors.
- a regional indicator alone is one (neutral East Asian Width); kitty and WezTerm draw it in two, wcwidth says two.
- spacing combining marks (Mc, 4358 code points: U+0903, U+093E, ...) are one column; wcwidth says zero. Not an
  emoji matter and not touched here, but it is the largest disagreement of all.
- a letter followed by U+FE0F is two columns (`a` + FE0F); VS16 only widens a base that has an emoji variation
  sequence, so most terminals draw one. Anchored (see `TUI_text.is_uncertain_cluster`), not corrected: that needs the
  list of emoji variation bases, which `text.unicode` does not expose.

**A pty harness is not in the repository**, so the terminal is the oracle only through `tests/TST_emulator.m31`, a
small emulator written for this (cursor moves, erase, the private modes, OSC 8 and OSC 52, a keyboard that sends
what the protocols say, two ways to measure text). The emulator found three real gaps in 0.9.1's handling of clusters of
uncertain width (see CHANGELOG), which byte-for-byte tests of the encoder could not. A `term`-level fake terminal (a
`Reader` over a byte list, a `Writer` into one) would let `Loop.run_with` itself be tested, not just its pieces; today
it is the one function that needs a tty, and `examples/features.m31` was run in a pty by hand.

**Small things.** No ternary (`a ? b : c`); `List<T>` has no `copy`; a parameter with a default can only be passed by
name (`clipboard_sequence(text, target: "c")`, a clear error); a local may not share a name with a function in the
module (`cluster_width`), which is clear too and cost a rename; `match` over another module's enum still cannot have a
`default` (§26), and the four-case match for `Event` is written out wherever an `Event` is passed on. The
full gate (seven programs, four compiler settings each) takes about four minutes.

## 29. 0.11.0: the after-frame hook, raw mouse, and what a cell costs (m31 v0.6.0)

Nothing here stopped the work.

- **An interface in an `Option` field works** (`pub Option<AfterFrame> after_frame = Option<AfterFrame>.None;`,
  set with `Option<AfterFrame>.Some(painter)` for a type that has the method and declares nothing), and a `match` on
  it needs no `default`. A function result may be ignored as a statement (`next.append_symbol(...)` now answers an
  `int`), so widening a `void` to a value is source compatible.
- **`str` has no substring search** (`contains` answers whether, not where), so a test that wants "the hook's bytes
  come before the cursor move" writes a `position` loop over `byte_at`. An `index_of` for `str` beside the one on
  `List` would do.
- **`m31c fmt`, `-r`, `--emit-c` and `-o` are deprecated in v0.6.0** (`-fmt`, `--recursive`, `-emit-c`, `--out`);
  `tests/test.sh` still uses the old ones, which print a warning, because CI also builds against v0.5.0.
- **A local may not be named like a function of the module** (§28) hit again: a test helper's loop variable `start`
  collided with `start(...)`. The message is clear.
- **What the 0.10.0 slowdown was.** `is_uncertain` was called for every cell the renderer wrote and began with two
  bounds-checked buffer reads (width, then size). Asking the size first, and not asking at all for a cell whose symbol
  `append_symbol` just reported as 3 bytes or fewer, removes both for nearly every cell. `full` on the 200x50
  benchmark went from 1.6-1.8 ms to 1.0-1.1 ms (0.9.1: 1.36 ms). Per-call overhead of a small public function is
  the cost to watch in m31: there is no inlining across modules that the author can rely on.
- **`os.exit(1)` from a failing test leaves `__rc_live=1`** (the live objects at exit), which reads like a leak next
  to the real failure; `test.sh` greps for it anyway.

## 30. Unreleased: where the time of a frame went, and what the compiler's share of it was (m31 v0.9.0)

Profiled with a sampling timer (SIGPROF at 2 kHz, instruction pointer and frame-pointer walk, symbolised with `nm`; there
is no `perf` here, and `gprof -pg` slowed the program 7x and put the cost on `mcount`) on the emitted C, gcc -O2.
On `TUI_diff.full` of the 200x50 widget screen, before this change: `rt_stack_check` 16%, `rc_dec` 14%, `rc_inc` 9%,
`Rect.has_point` 12% (it only exists to bounds-check), `Buffer.at` 8%; the work itself (`rt_bytes_push`) 3%. Three
quarters of a frame was the cost of calling small functions. Almost all of it is fixable in the library (a loop that
reads the lists once and writes bytes, instead of five calls per cell); what is left is the compiler's, and each item
below says where it shows in the C and what it is worth. The estimates are measured: the emitted C of the *old* renderer
and of the new one was patched (`sed` the line away, or `#define` the helper) and rebuilt, interleaved runs, medians,
on a machine that was loaded, so read them to within 15%.

1. **`rt_stack_check()` opens every emitted function and is a real call.** `fn_..._Buffer___at`, `Rect.has_point`,
   `Rect.right`: a three-line accessor is `call rt_stack_check` + its body. rt.h says why it is out of line (a
   thread-local read that gcc/clang caches across a green-thread switch). With the calls deleted, the old renderer's
   `full` went 2335 -> 1242 us (1.9x) and the finder keystroke diff 1003 -> 684 us (1.5x); on the new, loop-shaped code
   it is worth 0-13%. Win: 1.5-1.9x for call-heavy code. Possible fixes: no probe in a leaf (a function whose body calls
   nothing that can grow the stack or park the thread, which is every accessor above); or one probe per loop-free call
   tree instead of one per function.
2. **`rc_inc` and `rc_dec` are out-of-line calls in rt.c, and a field read passed as an argument is retained and
   released around the call.** `v3 = ((T32 *)v0)->f_box; rc_inc(v3); v4 = Rect_has_point(v3, x, y); rc_dec(v3);`
   (every `box.has_point(x, y)` in `Buffer`). The early-out for immortal objects (a str literal) is inside the call. A
   `static inline` fast path (`if (o->rc != RC_IMMORTAL) o->rc++`; for dec, decrement when the count stays above zero
   and the flags are clear, call out only to free) is worth 1.18x on the old `full`, 1.36x on the old paint, and 1.2x
   on the new diff. A field read that is only borrowed for the length of a call whose callee does not store it needs
   neither (the callee's parameter is borrowed already, as `v0` is).
3. **User types are heap objects behind a pointer, never value structs.** `Rect` is `{Obj hdr; int64 x, y, width,
   height}`, 48 bytes from `malloc`, and `box.y` is two dependent loads (`((T9 *)((T32 *)v0)->f_box)->f_y`).
   `TUI_geometry.Rect(x, y, 2, 1)` in `Buffer.set` is a malloc and a free. Rust has `Rect` inline in the parent and
   in registers. A `value type` (copied, no identity, no refcount) for small all-`int` structs would remove the
   allocation, the retain/release and one load level. Not measured as a patch (it is a language change); every `Rect`,
   `Style`, `Cell`, `Constraint` in the library is one.
4. **The hoisting pass works on locals and not on fields.** `while (i < end) { if (widths[i] != other.widths[i] ...` with
   `widths` a field re-reads `((T32 *)v0)->f_widths`, the `ty` tag, the length and the data pointer on every iteration
   (`rt_index_get`, 6 per cell). The same loop over `List<int> mine = cells;` is emitted with `rt_seq_data_addr` and
   `rt_len_of` once and `rt_index_get_at` in the loop (see `Buffer.ascii_run_end`'s C). The library now copies fields to
   locals by hand, which is a rule a programmer should not need to know. The pass could hoist a field read when nothing
   in the loop stores to that field or calls a function that might.
5. **Checked arithmetic and bounds checks.** `rt_iadd`/`rt_isub`/`rt_imul` for every index computation and loop
   increment (`v56 = rt_iadd(v37, 1)`), and `rt_index_get_at`'s range test on every element. With both removed, 1.16-1.19x
   on the old code, 1.0-1.1x on the new. Range-analysis would remove most: a `for (int i in 0 .. n)` counter cannot
   overflow, and `i` is inside `0 .. n` for the body. Already tracked; the loop form is the one place it is cheap to do.
6. **Everything together** (1, 2, 4's checks and 5): old renderer 2335 -> 785 us on `full` (3.0x), 1003 -> 542 us on
   the finder keystroke (1.85x), 591 -> 309 us on paint (1.9x). So against Rust, whose `Buffer` is a `Vec<Cell>` with
   no refcounts, no probes and no overflow traps, the ceiling "gcc compiles C" is real but the C we emit is not the C
   a person writes: it is 2-3x the instructions for call-shaped code, and the gap closes to about 10-20% once the
   library hands the compiler loops over locals. `-O3` helped the old code (1.4x on `full`) and hurt the new
   (0.82x), `-flto` helped both (1.14x, 1.3x on diff-0) but is excluded by the stack-probe rule in rt.h.
7. **`[v; n]` is the fast way to fill a list and nothing else is.** `rt_list_repeat` is one C loop; a list filled by
   `for ... push` is a call and a capacity check per element, and an indexed store loop is a mutability check and a
   range check per element. `clear()` of 10,000 cells went from 78 us to 8 us by assigning `[BLANK; n]`. A `fill(from,
   to, v)` and a `copy_within` on `List<int>` (a `memset` and a `memmove`) would let a buffer clear or scroll a region
   in nanoseconds, and `a.slice(i, j)` already does the copy half.
8. **`unicode.graphemes`, `unicode.width` and `str.chars()` allocate a list and a string per cluster.** After the
   renderer was fixed, 20-45% of what is left in a paint is `rt_list_push`, `str_new`, `malloc`/`free` and
   `class_of` from those (`Paragraph` wrapping, `set_str` of non-ASCII text). A cursor form (`for (int start, int end in
   text.cluster_bounds())`, no strings) and a `unicode.width_of_code_point(int)` would let `TUI_text` measure without
   building anything. This one is the standard library's, not the language's.
9. **`bytes.push` of a byte is a call into rt.c** (`rt_bytes_push`: capacity check, store, length). A frame writes 12-15
   thousand of them (9% of `full` after the rest was fixed). An inline fast path (`if (len < cap) data[len++] = v`)
   or `bytes.extend` from a slice of a `List<int>` would remove it.
10. **Module state would have let the buffers share one cluster table.** Two buffers compared cell by cell (previous and
    next frame) cannot share ids for multi-code-point clusters without an object both hold; the library keeps a table
    per buffer and compares clusters as strings, which is correct and rare. (Known, §14; not a request.)
