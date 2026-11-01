| scenario | m31 median us | m31 p95 | ratatui median us | ratatui p95 | m31 / ratatui | m31 bytes | ratatui bytes |
|---|---:|---:|---:|---:|---:|---:|---:|
| full-200x50 | 678.6 | 753.1 | 1257.8 | 1402.2 | 0.54x | 12358 | 15181 |
| diff-1cell | 459.4 | 509.5 | 777.4 | 862.4 | 0.59x | 19 | 47 |
| diff-0 | 449.2 | 479.2 | 767.4 | 833.2 | 0.59x | 0 | 25 |
| finder-ascii-key | 247.0 | 258.0 | 312.8 | 376.5 | 0.79x | 614 | 612 |
| scroll10k-diff | 367.4 | 390.1 | 485.2 | 559.2 | 0.76x | 1679 | 2421 |
| scroll10k-diff_scrolled | 510.5 | 540.6 | n/a | n/a | | 128 | |
| wide-full | 2325.6 | 2572.8 | 860.9 | 957.1 | 2.70x | 19547 | 20915 |
| wide-diff-1row | 1745.3 | 1894.6 | 509.8 | 578.1 | 3.42x | 348 | 45 |
| resize-storm-full | 467.3 | 571.3 | 907.0 | 1799.1 | 0.52x | 6071 | 13581 |
| paint-grid | 363.5 | 426.0 | 609.1 | 644.9 | 0.60x | 0 | 0 |
| paint-finder | 67.0 | 76.7 | 126.1 | 134.8 | 0.53x | 0 | 0 |

(median of the per-round medians, p95 of the per-round p95s; each round is 40 samples of ~10 ms)

| app (n runs) | start-up ms med / p95 | RSS KB med | idle CPU ms per s, med / p95 | key to frame ms med / p95 | resize to frame ms med / p95 |
|---|---:|---:|---:|---:|---:|
| ratatui (30) | 3.2 / 10.5 | 8432 | 0 / 0 | 0.4 / 0.9 | 0.5 / 1.0 |
| m31-probe-off (30) | 6.6 / 18.2 | 8448 | 15 / 50 | 0.4 / 1.2 | 100.5 / 101.5 |
| m31-default (30) | 155.2 / 168.5 | 8440 | 10 / 40 | 0.4 / 1.2 | 100.4 / 100.8 |
