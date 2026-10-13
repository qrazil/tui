# tui 0.6.0: syntax highlighting (`TUI_syntax`)

Version 0.6.0, from master 5327361 (v0.5.1). **Nothing was renamed or
removed**; this is an addition, so a program that depends on tui changes
nothing but its pin.

- New `core/TUI_syntax.m31`: `language_of(path, first_line)`, `languages()`,
  `classify(lang, line, state)` (a class per byte and the state the next line
  starts in: open block comments, triple-quoted strings, template literals,
  markdown fences), `is_dark(colorfgbg)`, `class_style`, `spans`, `draw`, and
  `Highlighter` -- lines fed in order, classified lazily and cached. It is the
  highlighter m3d1tor's `MED_syntax` grew, ported with its tests. Languages: c,
  cpp, go, javascript/typescript, json, m31, markdown, python, rust, sh, toml,
  yaml. Colours are the terminal's own 16, so they follow the user's scheme.
- `TUI_diff_view.DiffView` gains `language`, `is_dark` (and the private cache
  fields `syntax`, `syntax_language`). With `language` set, each hunk's code
  is coloured per token, classified along its old side and its new side
  separately and only as far as it is drawn; the marker column and the plain
  text of an added or removed line keep `add_style`/`remove_style`. With
  `language` empty (the default) the output is byte for byte what it was.
- `tests/syntax.m31` and `tests/syntax.out`: 110 checks, run by `tests/test.sh`
  with the other corpora.

# tui 0.5.0: the modules moved into folders

Version 0.5.0, from master 8edfea1 (v0.4.0). **No module, type, function or
field changed its name.** Only the file locations changed, so the only edit a
program that depends on tui has to make is the import path. Module names keep
the `TUI_` prefix and are still the name a program uses in code
(`TUI_buffer.Buffer`); there is still no namespacing.

| folder | modules |
|---|---|
| `core/` | `TUI_geometry`, `TUI_style`, `TUI_buffer`, `TUI_text`, `TUI_clip`, `TUI_widget`, `TUI_mouse`, `TUI_scroll`, `TUI_scale`, `TUI_eighths`, `TUI_word` |
| `widgets/` | `TUI_barchart`, `TUI_chart`, `TUI_diff_view`, `TUI_footer`, `TUI_gauge`, `TUI_input`, `TUI_jump_list`, `TUI_list_table`, `TUI_menu`, `TUI_outline`, `TUI_sparkline`, `TUI_spinner`, `TUI_tabs`, `TUI_textarea` |
| `app/` | `TUI_app`, `TUI_diff` |

In a program that lists `tui` in its `deps`, replace `import tui.TUI_x;` with
`import tui.<folder>.TUI_x;`, for example:

| old | new |
|---|---|
| `import tui.TUI_buffer;` | `import tui.core.TUI_buffer;` |
| `import tui.TUI_outline;` | `import tui.widgets.TUI_outline;` |
| `import tui.TUI_app;` | `import tui.app.TUI_app;` |
| `import tui.TUI_diff;` | `import tui.app.TUI_diff;` |

Inside the repository `core/` imports nothing from `widgets/` or `app/`, and
`widgets/` imports nothing from `app/`. Also changed: `scripts/check.sh` is now
`tests/test.sh` (the single entry point, same checks), the release tarball
carries `core/`, `widgets/`, `app/` and `RENAMES.md`, and it still needs
m31 v0.4.0. A dependent that pins tui by sha must re-pin and rewrite its imports
in the same commit. Tests and their expected output are byte-for-byte what they were.

---

# tui -> m31 0.4.0: what changes for a downstream program

Version 0.4.0, from master f19daf8 (v0.3.0). **No public name of tui changed in
this release**: every module, type, function, field, constant and named
argument keeps the name it had in 0.3.0, and `m31c lint` reports no finding
for this repository (it already followed `docs/naming-decision.md`). The
version is a minor bump because the library now needs **m31 v0.4.0 or later**
and does not build with older compilers (`import text.unicode;` is a path
import that only 0.4.0 knows), so a program on m31 0.3.x must stay on tui 0.3.0.

What a program that depends on tui has to do on moving to m31 0.4.0 comes from
the m31 standard library renames (m31's `CHANGELOG.md`, "Moving a program from
0.3.x"), because tui's public API passes m31 types through:

| where | old | new |
|---|---|---|
| import | `import unicode;` | `import text.unicode;` (the module is still used as `unicode.width(..)`) |
| `term.Size` returned by `term.size` and held by `TUI_app.Loop` code | `.cols` | `.columns` |
| `term.Key` handed to every widget's `handle` | `key.has(term.CTRL)` | `key.has_modifiers(term.CTRL)` |
| `fs` | `fs.is_dir(path)` | `fs.is_directory(path)` |

`term.Size.columns` and `term.Key.has_modifiers` are the two a handler written
against `TUI_app.Handler` is likely to meet; the other renames in the m31
changelog (`term.fg` -> `term.foreground`, `os.args` -> `os.arguments`, and
the rest) do not appear anywhere in tui's API.

Inside this repository: `TUI_app.m31` (window size fields), `TUI_input.m31`,
`TUI_textarea.m31` and `TUI_tabs.m31` (modifier tests), `TUI_buffer.m31` and
`TUI_text.m31` (the `unicode` import) and `examples/browse.m31`
(`fs.is_directory`). Tests and their expected output are untouched: the
output of `tests/tests.m31` and `tests/widgets.m31` is byte-for-byte what it
was.

---

# Earlier release (0.2.0), the naming convention renames

Branch `naming/migrate` of tui, from master 593d39a (v0.1.1). Consumers (gitui) must update every item below.

## Modules (files, imports, qualified uses)

| old | new |
|---|---|
| `tuiapp` | `TUI_app` |
| `tuibuf` | `TUI_buffer` |
| `tuidiff` | `TUI_diff` |
| `tuidiffview` | `TUI_diff_view` |
| `tuifooter` | `TUI_footer` |
| `tuigeom` | `TUI_geometry` |
| `tuijump` | `TUI_jump_list` |
| `tuilists` | `TUI_list_table` |
| `tuimenu` | `TUI_menu` |
| `tuioutline` | `TUI_outline` |
| `tuiscroll` | `TUI_scroll` |
| `tuistyle` | `TUI_style` |
| `tuitext` | `TUI_text` |
| `tuiwidget` | `TUI_widget` |

## Fields (break constructor calls with named arguments and every `.field` access)

| module | type | old | new |
|---|---|---|---|
| TUI_geometry | Rect | w | width |
| TUI_geometry | Rect | h | height |
| TUI_geometry | Layout | dir | direction |
| TUI_menu | Switch | on | is_on |
| TUI_style | Style | fg | foreground |
| TUI_style | Style | bg | background |

## Functions and methods

| module | old | new |
|---|---|---|
| TUI_style | Style.with_fg | Style.with_foreground |
| TUI_style | Style.with_bg | Style.with_background |

## Parameters (break only callers that pass the argument by name; positional calls are unaffected)

| module | function | old | new |
|---|---|---|---|
| TUI_app | resized | a | previous |
| TUI_app | resized | b | current |
| TUI_buffer | Cell.eq | o | other |
| TUI_buffer | Buffer.sized | w | width |
| TUI_buffer | Buffer.sized | h | height |
| TUI_buffer | Buffer.set | c | cell |
| TUI_buffer | Buffer.set_style | st | style |
| TUI_buffer | Buffer.set_str | s | text |
| TUI_buffer | Buffer.set_str | st | style |
| TUI_buffer | Buffer.fill | r | rect |
| TUI_buffer | Buffer.fill | s | symbol |
| TUI_buffer | Buffer.fill | st | style |
| TUI_buffer | Buffer.restyle | r | rect |
| TUI_buffer | Buffer.restyle | st | style |
| TUI_diff | diff | prev | previous |
| TUI_diff | show | b | data |
| TUI_diff_view | DiffView.render | buf | buffer |
| TUI_footer | Footer.render | buf | buffer |
| TUI_geometry | Rect.contains | px | point_x |
| TUI_geometry | Rect.contains | py | point_y |
| TUI_geometry | Rect.intersect | o | other |
| TUI_geometry | Rect.inset | m | margin |
| TUI_geometry | Rect.inset_xy | mx | margin_x |
| TUI_geometry | Rect.inset_xy | my | margin_y |
| TUI_geometry | Rect.eq | o | other |
| TUI_geometry | resolve | cs | constraints |
| TUI_geometry | apportion | wt | weights |
| TUI_geometry | rows | cs | constraints |
| TUI_geometry | cols | cs | constraints |
| TUI_jump_list | JumpList.sync_from | o | outline |
| TUI_jump_list | JumpList.follow | o | outline |
| TUI_jump_list | JumpList.activate | o | outline |
| TUI_jump_list | JumpList.render | buf | buffer |
| TUI_list_table | ListView.select | i | index |
| TUI_list_table | ListView.scrolled | h | height |
| TUI_list_table | ListView.render | buf | buffer |
| TUI_list_table | Table.render | buf | buffer |
| TUI_menu | WhichKey.render | buf | buffer |
| TUI_menu | Transient.render | buf | buffer |
| TUI_outline | Outline.render | buf | buffer |
| TUI_scroll | Viewport.reveal | at | index |
| TUI_scroll | Scrollbar.render | buf | buffer |
| TUI_style | Colour.from_code | c | colour_code |
| TUI_style | Colour.eq | o | other |
| TUI_style | Style.unpack | p | packed |
| TUI_style | Style.eq | o | other |
| TUI_style | Style.with_foreground | c | colour |
| TUI_style | Style.with_background | c | colour |
| TUI_style | Style.with_attrs | a | bits |
| TUI_style | push_int | v | value |
| TUI_style | sgr | to | target |
| TUI_style | palette_rgb | n | palette_index |
| TUI_style | rgb_to_256 | v | rgb_value |
| TUI_style | rgb_to_16 | v | rgb_value |
| TUI_text | width | s | text |
| TUI_text | wrap | s | text |
| TUI_text | truncate | s | text |
| TUI_text | truncate | ell | ellipsis |
| TUI_text | cut | s | text |
| TUI_text | drop_columns | s | text |
| TUI_text | pad | s | text |
| TUI_text | fit | s | text |
| TUI_text | fit | ell | ellipsis |
| TUI_text | offset | s | text |
| TUI_widget | Block.render | buf | buffer |
| TUI_widget | Paragraph.render | buf | buffer |
| TUI_widget | Line.render | buf | buffer |
| TUI_widget | Widget.render (interface) | b | buffer |
| TUI_app | Handler.handle (interface) | ev | event |

## Documentation-only renames

Positional variant payloads carry no name in the API, so their new labels appear only in docs and match bindings: `Rgb(r, g, b)` -> `Rgb(red, green, blue)`, `Indexed(n)` -> `Indexed(index)`, `Length(v)` -> `Length(cells)`, `Percentage(p)` -> `Percentage(percent)`, `Ratio(a, b)` -> `Ratio(numerator, denominator)`, `Min/Max(v)` -> `Min/Max(cells)`, `Fill(w)` -> `Fill(weight)`.

## Unchanged on purpose

Output formats (`Style.to_str` still prints "fg " and ", bg "), terminal escape sequences, and every string literal. Only trap messages changed, and only in their module prefix.

## Second pass: abbreviations and boolean names (lintv2)

Every entry below is public; gitui must update each. Boolean rule: a `bool` field, parameter or function name starts with is_/has_/can_/should_/did_/was_/needs_/will_.

### Fields

| module | type | old | new |
|---|---|---|---|
| TUI_buffer | Cell | sym | symbol |
| TUI_outline | Node | expanded | is_expanded |
| TUI_widget | Paragraph | wrap | should_wrap |

`Node.section(..., expanded: x)` becomes `is_expanded: x`; `Paragraph(..., wrap: x)` becomes `should_wrap: x`; `Cell(sym, ...)` is positional and unaffected.

### Functions and methods

| module | old | new |
|---|---|---|
| TUI_buffer | Buffer.sym_at | Buffer.symbol_at |
| TUI_buffer | Buffer.append_sym | Buffer.append_symbol |
| TUI_buffer | Buffer.inside | Buffer.is_inside |
| TUI_buffer | Buffer.same_cell | Buffer.is_same_cell |
| TUI_buffer | Buffer.same_row | Buffer.is_same_row |
| TUI_geometry | Rect.contains | Rect.has_point |
| TUI_geometry | cols | columns |
| TUI_app | resized | has_resized |
| TUI_jump_list | JumpList.activate | JumpList.did_activate |
| TUI_jump_list | JumpList.select_id | JumpList.did_select_id |
| TUI_outline | Outline.select_id | Outline.did_select_id |

### Parameters (named-argument callers only)

| module | function | old | new |
|---|---|---|---|
| TUI_buffer | Cell.of | sym | symbol |
| TUI_outline | Node.section | expanded | is_expanded |
| TUI_text | wrap, truncate, cut, drop_columns, pad, fit, offset | cols | columns |

### Private (no consumer impact)

Buffer fields sym/sty/wid -> symbols/styles/widths; TUI_diff `changed` -> has_changed; TUI_text `plain` -> is_plain; TUI_widget `Block.drawn` -> is_drawn; TUI_outline `Walk.visit` -> did_finish and `Seek.visit` -> did_find; TUI_buffer `ascii_run` -> is_ascii_run; TUI_menu `keyw` -> key_width; plus locals (running, redraw, grew, vis, cur_x/cur_y/cur_w, had, at_space, same_style, neg, cond, Sgr.first, back). Trap messages that quote a renamed method (`Buffer.symbol_at`, `Buffer.append_symbol`, `Buffer.is_same_cell`, `Buffer.is_same_row`) changed with it; test labels and other string literals did not.
