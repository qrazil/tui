# Changelog

Each version is a tag of github.com/qrazil/tui. `RENAMES.md` lists what a program has to change when an
API moves; this file says what each version added, changed and fixed. 0.9.0 and 0.9.1 were written back from
`git log` (beed45a, fdef5fb, 0b132ce); earlier versions are in `RENAMES.md` and the git history.

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
