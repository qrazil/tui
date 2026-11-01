| scenario | m31 median us | m31 p95 | ratatui median us | ratatui p95 | m31 / ratatui | m31 bytes | ratatui bytes |
|---|---:|---:|---:|---:|---:|---:|---:|
| full-200x50 | 673.4 | 770.4 | 1177.6 | 1377.8 | 0.57x | 12358 | 15181 |
| diff-1cell | 459.7 | 507.9 | 777.2 | 832.3 | 0.59x | 19 | 47 |
| diff-0 | 454.8 | 481.7 | 784.8 | 894.7 | 0.58x | 0 | 25 |
| finder-ascii-key | 232.4 | 244.0 | 325.8 | 374.0 | 0.71x | 626 | 602 |
| finder-cjk-key | 262.1 | 282.6 | 302.9 | 356.3 | 0.87x | 630 | 963 |
| scroll10k-diff | 326.2 | 345.8 | 469.9 | 517.2 | 0.69x | 1686 | 2445 |
| scroll10k-diff_scrolled | 489.4 | 512.2 | n/a | n/a | | 128 | |
| wide-full | 2012.5 | 2141.8 | 776.9 | 909.8 | 2.59x | 19547 | 20915 |
| wide-diff-1row | 1383.2 | 1550.0 | 484.6 | 568.2 | 2.85x | 348 | 45 |
| resize-storm-full | 468.8 | 631.0 | 858.4 | 1638.7 | 0.55x | 8946 | 13581 |
| paint-grid | 358.2 | 393.9 | 577.6 | 642.9 | 0.62x | 0 | 0 |
| paint-finder | 70.2 | 74.6 | 122.1 | 129.6 | 0.57x | 0 | 0 |
| links-diff-1cell | 479.8 | 525.0 | n/a | n/a | | 19 | |
| links-full | 718.8 | 782.6 | n/a | n/a | | 12436 | |
| wide-full-measured | 1937.0 | 2244.2 | (the wide frames with a measured, agreeing width table: bytes only fall, see the text) | | | 13955 | |
| wide-diff-1row-measured | 1387.7 | 1515.9 | (the wide frames with a measured, agreeing width table: bytes only fall, see the text) | | | 234 | |

(median of the per-round medians, p95 of the per-round p95s; each round is 40 samples of ~10 ms)

| app (n runs) | start-up ms med / p95 | RSS KB med | idle CPU ms per s, med / p95 | key to frame ms med / p95 | resize to frame ms med / p95 |
|---|---:|---:|---:|---:|---:|
| ratatui (30) | 3.8 / 5.2 | 8428 | 0 / 0 | 0.5 / 0.7 | 0.6 / 0.9 |
| m31-probe-off (30) | 5.7 / 6.3 | 8448 | 10 / 20 | 0.4 / 0.4 | 100.4 / 100.7 |
| m31-default (30) | 155.6 / 156.3 | 8372 | 10 / 20 | 0.4 / 0.4 | 100.4 / 100.6 |
