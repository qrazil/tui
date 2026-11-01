| scenario | m31 median us | m31 p95 | ratatui median us | ratatui p95 | m31 / ratatui | m31 bytes | ratatui bytes |
|---|---:|---:|---:|---:|---:|---:|---:|
| full-200x50 | 590.6 | 630.6 | 1091.3 | 1268.7 | 0.54x | 12358 | 15181 |
| diff-1cell | 414.6 | 450.1 | 735.0 | 812.6 | 0.56x | 19 | 47 |
| diff-0 | 405.9 | 455.7 | 710.9 | 1012.0 | 0.57x | 0 | 25 |
| finder-ascii-key | 208.3 | 237.3 | 291.6 | 331.2 | 0.71x | 623 | 605 |
| finder-cjk-key | 236.8 | 246.1 | 281.1 | 337.7 | 0.84x | 631 | 962 |
| scroll10k-diff | 300.5 | 317.4 | 416.8 | 483.6 | 0.72x | 1646 | 2424 |
| scroll10k-diff_scrolled | 454.7 | 466.8 | n/a | n/a | | 128 | |
| wide-full | 1767.5 | 2031.2 | 722.7 | 826.7 | 2.45x | 19547 | 20915 |
| wide-diff-1row | 1247.9 | 1385.9 | 459.5 | 546.8 | 2.72x | 348 | 45 |
| resize-storm-full | 469.3 | 583.4 | 777.9 | 1727.5 | 0.60x | 8149 | 13581 |
| paint-grid | 323.5 | 395.3 | 536.2 | 572.8 | 0.60x | 0 | 0 |
| paint-finder | 66.4 | 80.9 | 111.1 | 119.3 | 0.60x | 0 | 0 |
| links-diff-1cell | 434.1 | 489.5 | n/a | n/a | | 19 | |
| links-full | 642.8 | 788.0 | n/a | n/a | | 12436 | |
| wide-full-measured | 1837.2 | 2040.7 | (the wide frames with a measured, agreeing width table: bytes only fall, see the text) | | | 13955 | |
| wide-diff-1row-measured | 1359.0 | 1601.4 | (the wide frames with a measured, agreeing width table: bytes only fall, see the text) | | | 234 | |

(median of the per-round medians, p95 of the per-round p95s; each round is 40 samples of ~10 ms)

| app (n runs) | start-up ms med / p95 | RSS KB med | idle CPU ms per s, med / p95 | key to frame ms med / p95 | resize to frame ms med / p95 |
|---|---:|---:|---:|---:|---:|
| ratatui (30) | 3.3 / 10.7 | 8432 | 0 / 0 | 0.5 / 1.5 | 0.5 / 1.9 |
| m31-probe-off (30) | 5.7 / 25.9 | 8448 | 20 / 50 | 0.4 / 1.4 | 100.5 / 101.5 |
| m31-default (30) | 158.3 / 174.8 | 8348 | 20 / 30 | 0.5 / 2.2 | 100.5 / 100.9 |
