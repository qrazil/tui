# Changelog

Each version is a tag of github.com/qrazil/tui. `RENAMES.md` lists what a program has to change when an
API moves; this file says what each version added, changed and fixed. 0.9.0 and 0.9.1 were written back from
`git log` (beed45a, fdef5fb, 0b132ce); earlier versions are in `RENAMES.md` and the git history.

## 0.15.0

Speed: the renderer and the buffer were profiled on the emitted C (`docs/FRICTION.md` §30), and the cost was calls, not
cells -- five bounds-checked accessor calls per cell, each with a stack probe and a retain/release. The output is
byte-for-byte what 0.14.0 wrote (checked over the stress sequences, the finder and widget frames, and 1,500 random
frames of ASCII, CJK, emoji, combining marks, links and styles, in `full`, `diff`, `diff_scrolled` and
`diff_scrolled_in_place`); the public API is unchanged. On the 200x50 benchmark, interleaved medians against
0.14.0 on the same loaded machine: `full` 7-8x, one-cell and no-change diffs 1.7-1.9x, a finder keystroke (diff) 4-5.5x,
painting a finder frame 3.5-7x, `Buffer.clear` and `Buffer.sized` 8x.

### Changed

- **A cell is two integers, not three lists with a string.** `core/TUI_buffer.m31` stores `cells` (the glyph in the low
  32 bits, the width above) and `styles`. A glyph is the code point itself for a single-code-point cluster, 0 for a
  continuation, and an index into the buffer's own cluster table for a cluster of several; so writing a cell is two
  integer stores with no refcount traffic, and comparing two buffers is an integer compare (a string compare only for
  a multi-code-point cluster). The table is compacted when overwritten clusters outgrow the cells.
- **`TUI_diff.diff` and `full` pass over rows in runs.** New `Buffer` methods (`next_change`, `plain_run_end`,
  `changed_plain_run_end`, `append_run`) skip unchanged cells, find how far printable ASCII and characters of certain
  width run in one style, and write them as bytes in one pass. A cluster of uncertain width, a link, the erase tail and
  the forced window of a row still go through the per-cell code unchanged.
- **`fill`, `set_str` (printable ASCII) write the inside of a run without checking each cell** (only the ends of a run can
  split a wide character), `clear` and `sized` build their lists with `[v; n]`, `scrolled` of zero rows and `copy`
  copy the lists, and `set_str` measures a repeated cluster once. Strings of single non-ASCII code points that the
  renderer asks for are cached.

### Added

- `examples/bench_finder.m31`: the scenarios of the renderer comparison (full, diff-1, diff-0, and the keystrokes of a
  file finder in ASCII, CJK and emoji names, timed for diff, for paint and for both).
- `docs/FRICTION.md` §30: what the compiler's share of a frame was, each item with the emitted C and a measured estimate.

## 0.14.0

The owner of m3 draws in rio and Alacritty every day, wants Terminal.app and kitty to work, and asked for double-width
characters that break borders to be handled. 0.13.0 treated box drawing as certain (one column), so a terminal or profile that draws East Asian Ambiguous characters
wide ("ambiguous = wide", common with CJK locales and fonts) broke every border, and the harness listed it as a known limit. It
also distrusted every cluster whose width a terminal may measure differently, which is right and costs bytes on terminals that agree with
the library. 0.14.0 measures the terminal instead of guessing. Built and tested against m31 v0.9.0.

### Added

- **Start-up width calibration** (`TUI_width`, `Loop.calibrate_width`, run by `Loop.run_with`). After the alternate screen is entered, one write
  puts a reference character per class (box drawing, block elements, arrows, Greek and Cyrillic, shapes, Braille, the older emoji, a symbol
  with U+FE0F, a flag, a ZWJ family, a combining letter, a Nerd Font glyph; `A` and an ideograph as controls) each followed by a
  cursor position report request, and one read collects the answers: the column of the report is the width drawn. At most
  `Loop.width_probe_ms` (150) in all; standard output that is not a terminal, `Loop.should_calibrate_width = false` and `TUI_WIDTH_PROBE=off` skip it;
  a terminal that answers nothing, or answers the controls wrongly, keeps the 0.13.0 policy.
- **A per-session width table** (`TUI_width.Table`, `Loop.width_table`, `Buffer.table`). A class measured equal to the library's width is *certain*: its cells are
  diffed per cell like ASCII, with no forced rewrite and no re-anchoring (the 0.13.0 distrust cost on Alacritty, rio and kitty is
  gone: the stress finder sequence is 11 to 16 % smaller). A class measured differently is laid out at the measured width and its rows follow the 0.13.0
  policy. `Table.is_certain`, `columns`, `text_width`, `decision`, `width_of`.
- **Grapheme clustering (DEC mode 2027).** The same batch asks `CSI ? 2027 $ p`; a terminal that has the mode off and can switch it on is switched
  (`Loop.should_set_grapheme_mode`, default true), measured again with it on, and switched off on the way out (`leave_sequence`).
- **Border-safe output.** When box drawing is measured as other than one column, line drawing and block characters are written as ASCII
  by `Buffer.set_str`, `fill` and `set` (`TUI_width.ascii_for`), so `Block`, scroll tracks, chart axes and gauges keep their columns.
  `Loop.borders` (`Auto`, `Ascii`, `Unicode`), `TUI_BORDERS=ascii|unicode|auto` (ascii and unicode win over the field), and `Block.charset`
  (wins over both) choose it.
- **Replies never cost a key.** Events read while waiting for answers that are not answers (keys typed meanwhile, the replies to the loop's other questions)
  are kept in order (`Loop.take_replayed`) and read first. The questions are asked on row 2 because `CSI 1 ; 2 R` is Shift-F3. Late
  answers (`Loop.late_reports`) are dropped by `dispatch` instead of reaching the program as an unknown event.
- `ProbeLink` and `TermLink`: what the calibration talks to the terminal through, so a test (or a program with a loop of its own) needs no terminal.
- `tests/width.m31` (the classes, the replies, every terminal below, the failure modes, the borders, the fast path, the cost),
  `tests/width_pty.m31` with `tests/width_child.m31` (a real `Loop.run_with` on a pty, the test playing the terminal: answers, keys typed among them, answers held back,
  no answer at all, `TUI_WIDTH_PROBE=off`, `TUI_BORDERS`), emulator models of the owner's terminals (`TST_emulator.preset`: alacritty, rio, kitty, terminal-app,
  box-wide, a terminal with mode 2027, a mute one) that answer cursor reports and DECRQM by their own width model.
- `examples/width_probe.m31` prints what the calibration measured, class by class, and what the renderer decided, then the per-cluster table.

### Changed

- **`Loop.run_with` calibrates by default** (`should_calibrate_width` is true). It writes a batch of cursor position report requests after the alternate screen is entered and
  waits for the answers, at most 150 ms; a program that must not do that (or a terminal it cannot ask) sets `should_calibrate_width = false`, or the user sets
  `TUI_WIDTH_PROBE=off`. Nothing else about the loop's bytes changes on a terminal that answers: the frames are the 0.13.0 ones, minus the rewrites a measured class no longer needs.
- `tests/stress.m31` runs every terminal twice: with the terminal not asked (0.13.0) and calibrated, adds `preset alacritty`, `rio`, `kitty`, `mode-2027`, `box-wide`
  and `mute`, and asserts zero failures for every variant with the calibration on, `box-wide` and `no-combining` included (both were "known limits"); it reports the bytes
  of the finder sequence both ways and asserts a terminal that agrees with the library does not write more.
- `TUI_text.is_plain` is public.

### Known limits

- A terminal that answers the cursor report after the budget, or in pieces that arrive more than one idle read apart, may hand the program the first piece as an unknown event.
- The terminal is asked once. A font or locale changed while the program runs is not noticed.
- A class is measured by one reference character. A cluster of a class that the terminal draws differently from the reference (a newer emoji in a
  class the terminal knows only in part) is the reference's width; the older emoji are a class and the newer ones are not, for that reason.
- A cluster the terminal draws in more than two columns (a family is six on Alacritty) is a two-column cell, as before: the columns after it are written again.

### Cost

`bench` (200x50, in `examples/bench.m31`): `full`, `diff-1`, `diff-0` and `frame` are unchanged on a buffer with no table (12,555 bytes, 21, 0); on the
same screen with the table of an Alacritty-like terminal `full` is 12,542 bytes (the triangles are certain, so they are not re-anchored). Building the table from the
answers is 16 microseconds; the calibration, with the emulator on the other end, is about 0.12 ms (0.25 ms with the second batch for mode 2027); on a real terminal it is
the round trip of one write (under a millisecond locally).

## 0.13.0

Terminal corruption on real Macs (m3's fuzzy finder, mgit, tui-markdown) that every Linux test passed. The tests
asked an emulator that agrees with the library; real terminals disagree about wrapping at the last column, about
how wide some clusters are, and about what a write leaves behind. `tests/stress.m31` is the new harness: it draws
random frames (a text layer, a popup with a list that grows and shrinks, a query typed and erased, resizes) through the
real `Loop.frame_bytes`, replays the bytes on eighteen kinds of terminal and asserts, after every frame, that the
screen equals a from-scratch repaint on the same terminal (and the buffer itself where the terminal agrees with the
library on widths). Before the fix, 32 of the harness's checks failed; after it none do (see "Known limits" for what a renderer
cannot know). Built and tested against m31 v0.9.0.

### Fixed

- **A cluster drawn narrower than the library thinks left the old character in the column after it.** A VS16 emoji
  (`U+2764 U+FE0F`, `U+26A0 U+FE0F`), a keycap, a lone regional indicator, a new emoji the terminal's tables do not
  have: the diff skipped the continuation cell, so on Terminal.app and others the cell kept what the previous frame
  had there, and `full()` (Ctrl-L) had the same hole. A two-column cluster of uncertain width is now written over
  blanks in its own style (blank, `CUP` back, cluster), except when the previous frame is known blank there.
- **Autowrap.** `\e[?7l` is written on entry, with every `full()` frame, and undone (`\e[?7h`) on leaving. A cluster
  drawn wider than expected at the right margin used to wrap onto the next row, which the diff never rewrote; and
  where a terminal wraps at once, a write in the last column of the last row scrolled the screen. A narrow
  combining cluster in the last column keeps autowrap on for that one cell (a mark after a last-column cell joins the
  wrong cell with it off, on some terminals).
- **A wider notion of "uncertain width".** Only printable ASCII, precomposed Latin letters (U+00C0..U+024F without x and
  division), box drawing and blocks (U+2500..U+259F), CJK ideographs, kana, Hangul syllables and the fullwidth forms are
  trusted. Everything else (any emoji, Nerd Font glyphs, arrows, Greek and Cyrillic, halfwidth katakana, combining
  marks, Devanagari and Thai, jamo sequences) is a cluster whose width a terminal may measure differently.
  `TUI_text.is_uncertain_cluster` says so; `TUI_text.is_certain_width` is the allow-list.
- **A row with an uncertain cluster is written from that cluster to its end** whenever any cell of the row changes
  (`Buffer.first_unsure`): what a terminal leaves behind next to such a cluster depends on what was written before,
  and a row written whole from there is the same picture whatever the frames before it were. Rows without one are
  diffed per cell as before. The cursor is re-anchored with an absolute move after every uncertain cell and after any
  write that reaches the last column.
- **`full()` clears the screen first** (`\e[2J` after the pen reset): a repaint (first frame, resize, Ctrl-L, focus-in)
  no longer trusts what the terminal shows.
- Every changed row of a diff starts with the pen reset, so a style a terminal did not take cannot run into the next row.
- Invisible format characters (zero-width space, joiner or non-joiner on its own, direction marks, word joiner, BOM, bidi
  controls, line and paragraph separators) and a variation selector with no base are dropped by `Buffer.set_str`
  instead of being appended to the previous cell: some terminals draw them as a blank column.
- A failed write (`flush` error) makes the next frame a full repaint (`Loop.should_repaint`) instead of a diff against a
  screen that was never fully drawn.
- `Loop` no longer asks Apple's Terminal for synchronized output (`TERM_PROGRAM=Apple_Terminal`; it does not know mode
  2026 and does not answer; `TUI_app.is_apple_terminal`). `Loop.should_sync_on_apple_terminal` (default false) asks anyway, for a program
  whose user said so.

### Added

- `tests/stress.m31`, a `tests/test.sh` check of its own (about half a minute, run once): see above. The emulator
  (`tests/TST_emulator.m31`) models pending and instant wrap, DECAWM, width quirks (VS16 ignored or widening,
  ambiguous and private-use glyphs wide, new emoji narrow, jamo added up, combining marks in their own cell), erase with
  and without background colour, relative moves unclamped, sync ignored, scroll regions (DECSTBM, IND, RI, LF) and a
  terminal that ignores them, control characters obeyed and listed, and per-cell SGR.
- `examples/width_probe.m31`: run it in a terminal and it prints, per cluster, the width the terminal measured (cursor
  position report after printing it) against the library's, and whether the renderer treats the cluster as uncertain.
- `Loop.should_repaint_on_ctrl_l` (default false): CTRL-L writes the next frame in full (cleared screen, every cell), and the key is
  still handed to the program. mgit and tui-markdown had no way to repair a screen something else had written on.
- `TUI_style.detect_depth()` and `TUI_style.depth_of(colour_term, program, term_name)`: the colour depth from `COLORTERM`, `TERM_PROGRAM` and
  `TERM` (m3's rule, moved here). A program that never chose a depth wrote 24-bit colours (`38;2;r;g;b`) to Terminal.app, which has none
  before macOS 26 and misreads them as unrelated attributes (bold, underline, reverse, a background); mgit was such a program.
- `TUI_diff.AUTOWRAP_OFF`, `TUI_diff.AUTOWRAP_ON`, `Buffer.first_unsure`, `Buffer.is_plain_blank`,
  `TUI_text.is_certain_width`, `TUI_text.is_invisible`, `TUI_text.count_code_points`.

### Known limits (not fixable without asking the terminal)

- A terminal told to draw East Asian Ambiguous characters wide (an iTerm2 or Terminal.app preference) draws box-drawing
  characters two columns wide; the layout cannot be right, and the harness lists it as "known" (`box-wide`). Turn the
  preference off.
- A terminal that ignores DECSTBM breaks `should_scroll` (`no-decstbm`, "known"); no real one does. m3 turns the scroll
  optimisation off: it saved about 3 % of the bytes of a finder keystroke and cost more time than it saved.
- m31's `term` has no SIGWINCH: a resize is noticed when the loop next wakes. A window shrunk and grown again between
  two wake-ups is not seen. Ctrl-L repaints.
- A terminal that draws a combining mark in a cell of its own (`no-combining`) cannot be matched by a repaint.

### Cost

`bench`: diff-1 and diff-0 unchanged (21 bytes, 0.11 ms); `full` 0.92 ms to 1.2 ms and 12,533 to 12,555 bytes on the
200x50 benchmark screen (the extra work is the allow-list test of every non-ASCII cell and the clear). A diff of rows
that hold emoji costs a few dozen bytes more per row.

## 0.12.0

Found moving mgit onto `Loop.run_with`: the loop's read timeout is `min(tick_ms, time to the next tick)` and
`term.Reader.read` resolves a lone Escape only when a read times out, so under a ticker a lone Escape came out 1 to
250 ms after the key instead of after a fixed wait, and an escape sequence whose bytes arrived more than a few ms
apart (a slow ssh link) was read as Escape plus characters. tui-markdown's loop had a fixed `ESCAPE_WAIT_MS`
and mgit's 250 ms; `run_with` had neither. Built and tested against m31 v0.9.0.

### Added

- **`Loop.escape_wait_ms`** (default 0). Above 0 the loop reads with `term.Reader.read_keys(timeout, escape_wait_ms)`:
  a lone ESC byte, or a held start of `ESC [`, `ESC O` or an OSC sequence, is resolved as a key only after
  `escape_wait_ms` of NO FURTHER INPUT (counted from the last byte, restarted by each arrival). `tick_ms`,
  `animate_ms` and redraws do not shorten it, and the ticker keeps ticking on time while an Escape is waited out.
  Whole events are returned at once. Negative counts as 0.
- `Loop.read_next(reader, timeout_ms)`: the one read the loop makes.
- `tests/escape.m31` and `tests/escape_child.m31`: a pty test (the child is a real `Loop.run_with`) that writes ESC alone,
  `ESC [ A` split by 3 ms and 30 ms, `ESC` then `[ A` far beyond the wait, and a lone ESC under a 5 ms ticker.

### Default 0, and why

With `escape_wait_ms` 0 the loop calls `reader.read` exactly as in 0.11.0, so the bytes read and the timings of every
existing program are unchanged. A non-zero default is not provably safer: it would change when a lone Escape resolves
for every program (from "1 to `tick_ms` ms" to the default, which is later for an app that runs a 10 ms tick), and
only the program knows what its users' links need. So it is opt-in: 25 to 50 locally, more for slow links.
`read_timeout_ms` is unchanged.

### Changed

- **CI builds with m31 v0.9.0** (`M31_REF` in `.github/workflows/test.yml`, was v0.5.0): `Loop.escape_wait_ms` needs
  `term.Reader.read_keys`, which arrived in m31 v0.9.0. Building tui 0.12.0 needs m31 v0.9.0 or later.

## 0.11.0

Driven by what tui-markdown 0.6.0 found moving onto `Loop.run_with` (its `docs/DESIGN.md`, "Gaps hit in m31
and tui"). Additive: a `Loop()` with nothing new set writes exactly the bytes of 0.10.0 (checked against the
0.10.0 renderer by `tests/terminal.m31`: a diff, a full frame, with sync and with a cursor). Built and tested
against m31 v0.6.0.

### Added

- **An after-frame hook.** `interface TUI_app.AfterFrame { bytes after_frame(TUI_buffer.Buffer frame, bool
  is_full); }` and `Loop.after_frame` (an `Option<AfterFrame>`, `None` by default). It is asked once for every frame
  about to be written, after the diff is composed, and the bytes it answers are written in the same write, after
  the cells and before the cursor is put back, inside the synchronized update. For what is not a cell: OSC 66 scaled
  text, kitty graphics. `is_full` says every cell was rewritten, so such things have to be sent again. A frame
  with no changed cell is still written when the hook answers bytes. `Loop.frame_bytes(previous, next, is_full,
  shown)` is the frame `run_with` writes (public, so it can be tested without a terminal) and
  `Loop.compose_frame_with(body, extra, shown)` the composition; `compose_frame` is it with no extra bytes.
- **Raw mouse.** `Loop.should_report_raw_mouse`: the mouse (and with `should_report_hover`, motion) modes are
  written on entry and taken off on the way out, and each report reaches `on_event` as an `Event.Mouse`, with no
  gesture routing and no `on_mouse`. `should_report_mouse` still routes, and wins when both are set.
- `Buffer.append_symbol` answers how many bytes it appended.

### Changed

- `TUI_diff.is_uncertain` asks for the symbol's size before anything else, and `full` and `diff` ask it only of
  cells whose symbol is longer than 3 bytes (`MAX_CERTAIN_BYTES`): a letter, a Latin or Hangul or CJK character or
  a box is never uncertain. Same answers (the 3773-sequence emoji test and the emulator oracle are unchanged); the
  0.10.0 emoji-width fix had made `full` slower by calling it for every cell written.
- The README's tui-markdown mapping table has rows for the after-frame hook and for the raw mouse.

### Decided against

- **OSC 66 in the cell model.** tui-markdown draws a scaled heading as a rectangle of cells that are blank plus
  an overlay (`MDV_render.Region`: x, y, width, scale, text, style) that a later write may clear and that has to be
  sent again after a scroll or a repaint. A cell model for it would need a cell that owns a w x h block (a
  continuation in two dimensions), a rule for what overwriting any one cell of the block does, and a diff that
  compares and re-sends blocks: more than the whole of this release, for one protocol. The hook is the same size
  as the problem and also serves kitty graphics. If a second program needs scaled text, revisit.

## 0.10.0 (unreleased)

Terminal features, each opt-in, each tested against a terminal emulator (`tests/TST_emulator.m31`). Defaults
keep today's bytes on the wire: a `Loop()` writes what it wrote in 0.9.1 and a frame with no hyperlink is
byte for byte the same. Built and tested against m31 v0.6.0 (the new test program and the old ones also compile
and pass against v0.5.0, which CI pins).

### Added

- **Synchronized output**, composed in one place: `Loop.compose_frame(body, shown)` and
  `TUI_diff.synchronized(body)`, usable from a loop of your own.
- **OSC 52 clipboard write**: `TUI_osc` (`clipboard_sequence`, `clipboard_fit`, `CLIPBOARD_LIMIT` 74994),
  `Loop.copy(text)`, `Loop.is_copy_allowed` (the off switch), `Loop.copy_limit`, and the queue behind it
  (`Loop.queue_output`, `Loop.take_output`), written before the next frame and on the way out of a `Quit`.
  No encoder for the OSC 52 read.
- **OSC 8 hyperlinks** in the cell model: `Cell.link`, `Buffer.link(rect, uri)`, `set_link_str`, `link_at`,
  `has_links`, `symbol_size_at`, `TUI_widget.Span.link`, `TUI_osc.hyperlink_uri` / `hyperlink_open`; `TUI_diff`
  writes, closes and compares them. A buffer that never had a link pays nothing.
- **Keyboard protocols**: `TUI_keys`; `Loop.should_disambiguate_keys` (kitty, flag 1, asked with `CSI ? u` and
  `CSI c`, pushed only when answered), `Loop.should_try_modify_other_keys` (xterm level 2, as the fallback or
  alone), `Loop.key_protocol`, `start_keys`, `keys_off_sequence`, `did_take_key_reply`, `translate`, and
  `Loop.dispatch` is public. Shift-Tab is `BackTab` under either protocol.
- **Focus**: `Loop.focus_changes`, `Loop.should_repaint_on_focus`, `TUI_app.focus_report(bytes)`.
- `Loop.input_fd`, `TUI_app.INPUT_AUTO`, `TUI_app.pick_input_fd` (read keys from the tty standard output is when
  standard input is a pipe: what `tui-markdown` needs to use `run_with`).
- `TUI_text.cluster_columns`, `is_uncertain_cluster`, `uncertain_reach`; `TUI_diff.uncertain_reach_at`.
- `examples/features.m31`, `tests/terminal.m31`, `tests/TST_emulator.m31`, `tests/emoji_sequences.txt`.
- README: the 0.10.0 section, including the notes for adopting `Loop.run_with` from a loop of your own.

### Changed

- `leave_sequence()` begins with `CSI ? 2026 l` when `should_sync` is set (a process killed in the middle of a
  frame no longer leaves the terminal holding its screen), and with the keyboard reset when a protocol is on.
- The loop's own answers (the mode 2026 reply, the keyboard replies) return `Redraw` (the mode 2026 one was `Continue`), so a
  view that shows `is_sync` or `key_protocol` is drawn again once the terminal has answered. A frame that
  changes nothing writes nothing.
- After a cluster of uncertain width the renderer now rewrites as many columns as a terminal that adds up code
  points may have overdrawn (`TUI_text.uncertain_reach`: a family 8, England 12, a flag 2) instead of always two.
  A frame holding such a cluster writes a few more bytes when it changes; no such cluster, no change.
- `Loop.Shown` is public (it is the argument of `compose_frame`).

### Fixed

- A skin tone after a text-presentation base (a raised hand, a victory hand...: 45 of the 3773 fully-qualified
  sequences of Unicode emoji-test.txt 15.1) was measured as one column; it is two. A variation selector on its
  own is zero columns (it was two).
- Not anchored, so drifting on some terminals: a lone regional indicator, a letter with U+FE0F, a joined
  sequence one column wide, a skin tone on a text base. They are clusters of uncertain width now.

## 0.9.1 (2026-10-16)

- A cell holding a control character was written to the terminal as it was (a tab moved the cursor, CR returned
  it, ESC started a sequence). `set_str`, `set` and `fill` draw the Control Picture (U+2400 + code, U+2421 for
  DEL), or U+FFFD for a C1 control, in its place.
- Clusters of uncertain width (an emoji ZWJ sequence, a flag, a variation-selector emoji: two columns where
  grapheme clusters are segmented, more where code points are added up): `TUI_diff` puts the cursor back with an
  absolute move after one, never walks across one, and writes the two columns after it again.
- `Loop.should_repaint` writes the next frame in full (a focus-in report sets it): the way to repair a screen
  something else wrote on.

## 0.9.0 (2026-10-15)

- `TUI_diff.diff_scrolled` / `diff_scrolled_in_place`: a view that moved by a few lines is scrolled with a
  terminal region instead of rewritten; `Buffer.scrolled`, `scroll_rows`, `copy`, `is_row_like`,
  `is_blank_row` support it.
- `TUI_app.Loop`: synchronized output (DEC 2026, asked with DECRQM, used only when the terminal answers:
  `should_sync`, `is_sync`); the real cursor and its shape (`cursor`, `should_manage_cursor`); focus reports
  (`should_report_focus`, `has_focus`); `is_input_queued`; `frame_count`, `last_frame_bytes`, `last_frame_us`;
  `animate_ms` may change from a callback; `drain_events`; the clock is `timer.monotonic_ns` (needs m31 0.5.0).
- CI builds against m31 v0.5.0.
