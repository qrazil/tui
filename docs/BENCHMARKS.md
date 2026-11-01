# Benchmarks: tui against ratatui

What a frame costs this library, what it costs the same frame in [ratatui](https://github.com/ratatui/ratatui)
(Rust), what each writes to the terminal, how an idle application behaves, what the review of 0.16.0 changed, and what
is left, ranked. The tui-markdown against glow comparison is in tui-markdown's `docs/BENCHMARKS.md`.

Everything here is reproducible: `bench/` holds both programs, the runner and the raw data.

```sh
M31_ROOT=/path/to/m31 LANGC=/path/to/m31c bash bench/run.sh [rounds=3] [idle_runs=30]
```

It builds `bench/bench.m31` and `bench/idle.m31` with m31, builds `bench/ratatui` (ratatui 0.30.2, crossterm backend,
`Cargo.lock` committed, built in a temporary target directory, release with LTO), runs both alternately pinned to one
core, and writes `bench/results/<label>/` (`render-*.txt`, `idle-*.txt`, `machine.txt`, `summary.md`).
`python3 -I bench/summarize.py <dir>` rebuilds the tables below from the raw files.

## Method

- **What is measured**: the cost an application pays per frame -- clear the buffer, lay the frame into it, diff it
  against the last one, serialise to bytes -- and nothing a terminal does with the bytes. m31: `Buffer.clear`, `set_str`,
  `fill`, `Block.render`, `TUI_diff.diff`/`full`. ratatui: `Terminal::draw` with a `CrosstermBackend` over a
  `BufWriter` into a counting sink (so `bytes` is what the terminal would be sent). ratatui's `Backend` is wrapped only to
  answer the three things it asks a tty (cursor position, size, window size) with constants.
- **Sampling**: a sample is the mean of `reps` frames (`reps` chosen so a sample lasts about 10 ms); 5 samples are
  thrown away, 40 kept; the table gives the median and the p95 of the 40, and over 3 rounds (processes) the median of the
  medians and of the p95s (so 3 rounds of 40 samples per cell).
- **Machine**: Intel Core i7-8750H (2.2 GHz base, turbo), 12 threads, Linux 6.14 (Fedora 40), gcc 14.2 -O2 for the
  m31 output, rustc 1.97.1 `--release` with LTO and `codegen-units = 1`; the `powersave` governor (it could not be
  changed without root) and a **load average of 2-5 from other work on the machine**: absolute microseconds move 10-25 %
  between runs of the same binary (see the 0.15.0 and 0.16.0 rows of `diff-0` below, which did not change), so compare the
  *ratio* of two programs run alternately, and treat an A/B of two builds of the library as valid only when interleaved
  (`bench/results` has the raw runs; the commit messages of 0.16.0 give interleaved before/after numbers).
  The benchmark pins itself to one core with `taskset`; frequency scaling is not controlled.
- **Compilers**: m31 v0.9.0 (the toolchain tui is tested with) and v0.12.0 (the current release), both from the public
  release assets with their checksums verified.
- **Scenarios** (identical frames in both programs, `bench/bench.m31` and `bench/ratatui/src/main.rs`):

| scenario | what |
|---|---|
| full-200x50 | 8 coloured text segments per row on every row (mixed text and styled cells), repainted from a cleared screen |
| diff-1cell | the same frame with one cell toggled between frames |
| diff-0 | the same frame, nothing changed (what an idle redraw costs) |
| finder-ascii-key | a ruled box, a query row and forty list rows with the first selected; a keystroke changes the query and the list (about 10-40 rows change) |
| finder-cjk-key | the same with CJK names |
| scroll10k-diff | a 10,000-line buffer, the 50-row window moving down one line per frame, diffed (ratatui has no scroll optimisation); `scroll10k-diff_scrolled` is m31's `diff_scrolled_in_place` (a scroll region, 128 bytes) |
| wide-full, wide-diff-1row | every row holds CJK, kana, Hangul, emoji, a ZWJ family, a VS16 heart; full, and one cell of one row changing |
| resize-storm-full | eight window sizes in turn, each a new buffer, a paint and a full redraw |
| paint-grid, paint-finder | laying the frame into a buffer only (no diff, no bytes) |
| links-* | (m31 only: ratatui has no hyperlinks) one hyperlink in the 200x50 frame |

For the empty application (`bench/idle.m31`, `bench/ratatui/src/bin/idle.rs`, driven by `bench/idle.py` over a pty of
200x50): start-up to the first frame, peak RSS, CPU used in one idle second, key to the frame that shows it (5 keys),
SIGWINCH to the frame in the new size (4 resizes), 30 runs each.

## Results (m31 v0.9.0, tui 0.16.0)

`bench/results/v0.16.0-m31-v0.9.0/summary.md`. Microseconds per frame; the ratio is m31 / ratatui (below 1: m31 is faster).

| scenario | m31 median us | m31 p95 | ratatui median us | ratatui p95 | m31 / ratatui | m31 bytes | ratatui bytes |
|---|---:|---:|---:|---:|---:|---:|---:|
| full-200x50 | 673.4 | 770.4 | 1177.6 | 1377.8 | 0.57x | 12358 | 15181 |
| diff-1cell | 459.7 | 507.9 | 777.2 | 832.3 | 0.59x | 19 | 47 |
| diff-0 | 454.8 | 481.7 | 784.8 | 894.7 | 0.58x | 0 | 25 |
| finder-ascii-key | 232.4 | 244.0 | 325.8 | 374.0 | 0.71x | 626 | 602 |
| finder-cjk-key | 262.1 | 282.6 | 302.9 | 356.3 | 0.87x | 630 | 963 |
| scroll10k-diff | 326.2 | 345.8 | 469.9 | 517.2 | 0.69x | 1686 | 2445 |
| scroll10k-diff_scrolled | 489.4 | 512.2 | n/a | n/a | | 128 | |
| wide-full | 2012.5 | 2141.8 | 776.9 | 909.8 | **2.59x** | 19547 | 20915 |
| wide-diff-1row | 1383.2 | 1550.0 | 484.6 | 568.2 | **2.85x** | 348 | 45 |
| resize-storm-full | 468.8 | 631.0 | 858.4 | 1638.7 | 0.55x | 8946 | 13581 |
| paint-grid | 358.2 | 393.9 | 577.6 | 642.9 | 0.62x | 0 | 0 |
| paint-finder | 70.2 | 74.6 | 122.1 | 129.6 | 0.57x | 0 | 0 |
| links-diff-1cell | 479.8 | 525.0 | n/a | n/a | | 19 | |
| links-full | 718.8 | 782.6 | n/a | n/a | | 12436 | |
| wide-full-measured | 1937.0 | 2244.2 | | | | 13955 | |
| wide-diff-1row-measured | 1387.7 | 1515.9 | | | | 234 | |

(`-measured`: the same wide frames with the width table of a terminal that agrees with the library, what
`Loop.calibrate_width` makes of Alacritty, kitty or rio; bytes fall by 29 % and 33 %, time does not move because the
frame is dominated by laying text into the buffer.)

With m31 v0.12.0 (`bench/results/v0.16.0-m31-v0.12.0/summary.md`) the m31 column is 5-12 % lower and the ratios are
0.54-0.60 on the ASCII scenarios and 2.45x / 2.72x on the wide ones: the compiler release is a little faster and nothing
else changes.

| app (30 runs each) | start-up ms med / p95 | RSS KB | idle CPU ms per second, med / p95 | key to frame ms med / p95 | resize to frame ms med / p95 |
|---|---:|---:|---:|---:|---:|
| ratatui | 3.8 / 5.2 | 8428 | 0 / 0 | 0.5 / 0.7 | 0.6 / 0.9 |
| tui, calibration off (`TUI_WIDTH_PROBE=off`) | 5.7 / 6.3 | 8448 | 10 / 20 | 0.4 / 0.4 | **100.4 / 100.7** |
| tui, default (calibration on, the pty answers nothing) | 155.6 / 156.3 | 8372 | 10 / 20 | 0.4 / 0.4 | **100.4 / 100.6** |

The 155 ms is the 150 ms `width_probe_ms` wait on a terminal that never answers the cursor-position requests (a pty
under a script); a real terminal answers in about a millisecond and the calibration costs a round trip.

## Reading the numbers

**Speed of a frame.** On ASCII and mixed-style frames the library is 0.55-0.75x of ratatui's time and writes fewer bytes
(a one-cell diff is 19 bytes against 47, a full redraw 12.4 KB against 15.2 KB, a scroll 128 bytes against 2.4 KB with
`diff_scrolled`). Two reasons, only partly measured: the renderer skips unchanged rows with one pass over two integer
lists and writes runs of equal-style ASCII as bytes (0.15.0), and ratatui lays each string into the buffer by
`unicode-segmentation` graphemes with a `String` per cell (its `paint-grid` and `paint-finder` are 1.6-1.8x ours). The
diff-0 cost (455 us) is almost all painting (358 us): the diff of 10,000 unchanged cells is about 100 us.

**Where tui loses, and why (the cost model).**

1. *Wide characters and emoji, 2.5-2.9x slower.* `set_str` of a text with anything but Latin, ideographs, kana, Hangul
   and fullwidth forms takes the cluster path: `unicode.graphemes` allocates a list and a string per cluster,
   `Table.columns` calls `class_of`, `glyph_of` builds strings. A sampling profile of `wide-diff-1row` (the diff itself
   is a small part of it) is `class_of` 10 %, `rc_dec` 8 %, `rt_list_push` 4 %, `rt_stack_check` 4 %, `unicode.cluster_starts`
   4 %, `str.chars` 2 %, with a third in libc `malloc`/`free`/`memcpy`. This is FRICTION section 8 (the standard library's
   per-cluster allocation) and sections 1-2 (a probe and a retain/release around every small call). 0.16.0 takes the
   common CJK and Latin case off it (`finder-cjk-key` 375 -> 224 us; this frame has emoji and ZWJ in every row, which keep
   the path, 1.2-1.3x).
2. *Bytes of a wide diff, 348 against 45.* Not the compiler: a policy. A cluster whose width terminals may disagree on
   (an emoji, a flag, a sequence, a symbol with a selector) makes the renderer write the rest of its row again, with an
   absolute move after it, so that a terminal that measures it differently cannot leave the row wrong (0.13.0, 0.14.0;
   the stress harness replays 26 kinds of terminal). On a terminal whose measured table agrees it is 234 bytes
   (`-measured`); ratatui trusts `unicode-width` and writes the one cell. The renderer could trust more classes once measured
   (the newer emoji are in no class and stay uncertain) -- see the list below.
3. *Idle CPU, 10-20 ms a second against 0.* Not tui's loop: a program that only calls `timer.sleep_ms(3000)` takes about 940
   voluntary context switches a second (measured on v0.9.0 and v0.12.0, `/proc/<pid>/status`), so the runtime's scheduler
   wakes about every millisecond with every green thread parked. Compiler request 1 in `docs/FRICTION.md` section 31.
4. *A resize is noticed 100 ms late, not at once.* m31 has no way to be told of SIGWINCH (`runtime/sys.h` says so), the loop
   asks for the size after every read, and the latency is a constant 100.4 ms (median and p95 alike, with the loop's `tick_ms` at its default of 250), which
   points at a 100 ms slice inside `term.Reader.read` (the source of `term` is in the compiler, not in this repository). ratatui gets an event from
   crossterm's signal handling. Compiler request 2.
5. *Start-up, 5.7 against 3.8 ms*, RSS equal (8.4 MB both: a runtime with a thread pool against a Rust binary). Not a loss that matters; the
   calibration's 150 ms ceiling is only paid by a terminal that does not answer.

Against FRICTION section 30's causes: the gap on ASCII is gone (the loops hand the compiler locals), so causes 1, 2, 4 and 5
(probes, retain/release, field hoisting, checked arithmetic) show only in the cluster path and in anything call-shaped; cause 8
(string-per-cluster) is the one that now sets the wide-text cost.

## What changed in 0.16.0 (interleaved A/B against 0.15.0, m31 v0.9.0, one core, medians)

| change | scenario | before | after |
|---|---|---:|---:|
| link flags per row (`link_rows`) | links-full | 1893 us | ~640 us (2.9x) |
| | links-diff-1cell | 579 us | ~410 us (1.4x) |
| `set_str` from code points (`simple_width`) | finder-cjk-key | 375 us | 224 us (1.67x) |
| | wide-diff-1row | 1470 us | 1135 us (1.3x) |
| | wide-full | 1920 us | 1615 us (1.19x) |
| `hyperlink_uri` fast path | `hyperlink_open`, 90-byte URL | 5709 ns | 619 ns (9.2x) |

Bytes are identical (a hash over 4800 random frames, and every `tests/*.out`). The 0.15.0 and 0.16.0 rows of `bench/results`
agree with this within the machine's noise.

## Review findings that are not speed

Fuzzing (`tests/fuzz.m31`, and by hand on pipes and a pty): the input decoder (`term.decode`) never trapped on 6000
random byte strings, is linear (a 4 MB bracketed paste decoded in 70 ms through a pipe, one `Paste` event), and a UTF-8
character, a CSI and a paste split across reads arrive intact (checked with `term.Reader` on a pipe written in pieces,
300 ms apart). An unterminated paste comes out as one `Unknown` event at end of input. Junk SGR mouse reports decode to
odd coordinates and button masks without trapping. Hostile text (ESC, CSI, OSC 52, NUL, DEL, C1, newline, bidi override,
a ZWJ sequence broken by a control) never reaches a terminal as a control: it is drawn as a control picture and the
emulator obeys nothing. A URI with controls is `%XX`-escaped before OSC 8. **No bug was found**; one defect of cost was
(a frame with one hyperlink falling off the fast path, fixed).

Exit paths (measured on a pty with `bench/idle.m31`): `q` and a normal return restore the terminal (alternate screen off,
cursor shown, autowrap on). SIGTERM, SIGINT and SIGHUP end the process with the alternate screen, hidden cursor and
mouse modes **left on** (nothing is written on the way out): the known gap of FRICTION section 23, still present in
v0.12.0, compiler request 3. Ctrl-C in raw mode is a key (`ctrl+c`), not a signal, so it reaches the program.

## Perf updates we should have

Ranked by value for an application, with what stands in the way.

**Done in 0.16.0**

1. Row-level link flags: a page with a link is no longer 3x slower to draw.
2. `set_str` of Latin, CJK, kana, Hangul, fullwidth text from code points.
3. `hyperlink_uri` without per-byte strings.
4. The suite itself, and `tests/fuzz.m31` as a gate.

**Worth doing (library, not risky, not done yet)**

1. *Wide-text paint*: cache the cluster layout of a string (clusters, widths) per buffer, keyed by text, for the labels
   an application repaints every frame, and extend the code-point path to runs of simple text around an emoji. Not done:
   the cache's validity depends on the width table (assigned to a buffer after creation), so it needs `Buffer.table`
   to become a method that clears it. Expected: the wide scenarios from 2.6x toward 1.5x of ratatui.
2. *Trust measured classes*: with a calibrated table, treat the newer emoji and sequences as certain when the same
   probe family measured agreeing (needs more probe characters in the batch) -- the wide diff from 234 bytes toward 45.
3. *`TUI_text.cut` and `truncate` are O(length of the text)*, not O(columns): `cut` builds the cluster list of the
   whole string before taking a prefix (measured 32 ms for a 280 KB line cut to 70 columns, per call). Cut at a safe byte
   prefix first. A long line in a list is a per-frame cost.
4. *Skip blank rows in `full`* after `\e[2J`: 9 bytes a row (about 450 bytes on a mostly empty screen). It changes the bytes of
   the first frame, so tui-markdown's pty golden (`tests/golden/pty_default_0.9.0.txt`) has to be re-blessed with it.
5. *`REP` (`CSI n b`) for runs of one character* and *`CUD`/`CUF` instead of `CUP`* when shorter: gated on the calibration
   (kitty, Alacritty, xterm and rio have REP; Terminal.app does not), a few percent of bytes on rules and fills.
6. *A damage API*: `Loop.mark_dirty(Rect)` so an application that knows what it touched skips `clear` and the
   whole-frame paint; worth it only above 200x50 or on slow paints.
7. *Resize*: `tick_ms` (250) is not what bounds it (the 100 ms is below it); nothing to do until the compiler offers SIGWINCH.
8. *Scripts*: `scripts/build.sh`, `tests/test.sh` use `--emit-c`/`-o`/`fmt`, which v0.12.0 reports as deprecated.

**Compiler-bound** (requests with evidence in `docs/FRICTION.md` section 31)

1. An idle program wakes about 1000 times a second (10-20 ms CPU per second for an empty TUI; ratatui: 0).
2. No resize notification (SIGWINCH): 100 ms late.
3. A signal ends the process without writing `Session.on_exit`: alternate screen, mouse and hidden cursor are left on.
4. Per-cluster allocation in `unicode.graphemes`, `unicode.width`, `str.chars` (section 8): the wide-text cost.
5. Value structs, inline `rc_inc`/`rc_dec`, probe-free leaves, range-checked loop arithmetic (section 30): the remaining 10-20 %
   of the ASCII path and 2-3x of any call-shaped code.
6. `bytes.push` inline fast path (section 30.9).

## Raw data

`bench/results/v0.15.0-m31-v0.9.0`, `v0.15.0-m31-v0.12.0` (before the review) and `v0.16.0-m31-v0.9.0`, `v0.16.0-m31-v0.12.0`:
`render-m31.txt` and `render-ratatui.txt` (a `RESULT <name> <median_us> <p95_us> <min_us> <bytes>` line per scenario per
round), `idle-*.txt` (a `RUN` line per process), `machine.txt`, `summary.md`. The 0.15.0 directories do not have the
`links-*`, `finder-cjk-key` and `-measured` scenarios, which were added after; their before-numbers are the interleaved A/B
above. `bench/idle.py` reads `/proc` and uses a pty, so it is Linux-only; the render benchmark runs anywhere with a C
compiler and cargo.
