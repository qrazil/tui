| scenario | m31 median us | m31 p95 | ratatui median us | ratatui p95 | m31 / ratatui | m31 bytes | ratatui bytes |
|---|---:|---:|---:|---:|---:|---:|---:|
| full-200x50 | 553.2 | 633.1 | 1061.3 | 1221.1 | 0.52x | 12358 | 15181 |
| diff-1cell | 379.4 | 414.1 | 666.2 | 732.3 | 0.57x | 19 | 47 |
| diff-0 | 373.9 | 393.5 | 641.3 | 698.2 | 0.58x | 0 | 25 |
| finder-ascii-key | 202.7 | 217.4 | 268.4 | 292.4 | 0.76x | 626 | 612 |
| scroll10k-diff | 313.0 | 324.4 | 406.5 | 486.3 | 0.77x | 1686 | 2458 |
| scroll10k-diff_scrolled | 428.0 | 449.9 | n/a | n/a | | 128 | |
| wide-full | 1970.2 | 2235.9 | 707.1 | 826.7 | 2.79x | 19547 | 20915 |
| wide-diff-1row | 1483.9 | 1599.1 | 424.0 | 481.4 | 3.50x | 348 | 45 |
| resize-storm-full | 372.0 | 439.8 | 773.3 | 1493.7 | 0.48x | 6071 | 13858 |
| paint-grid | 313.6 | 337.7 | 499.6 | 528.1 | 0.63x | 0 | 0 |
| paint-finder | 56.7 | 61.6 | 108.0 | 113.2 | 0.52x | 0 | 0 |

(median of the per-round medians, p95 of the per-round p95s; each round is 40 samples of ~10 ms)

| app (n runs) | start-up ms med / p95 | RSS KB med | idle CPU ms per s, med / p95 | key to frame ms med / p95 | resize to frame ms med / p95 |
|---|---:|---:|---:|---:|---:|
| ratatui (30) | 4.7 / 11.4 | 8382 | 0 / 0 | 0.8 / 1.5 | 0.8 / 2.2 |
| m31-probe-off (30) | 6.1 / 16.7 | 8042 | 10 / 30 | 0.4 / 1.2 | 100.4 / 101.2 |
| m31-default (30) | 168.1 / 168.8 | 8448 | 40 / 50 | 1.1 / 1.3 | 100.7 / 101.6 |
