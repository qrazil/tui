#!/usr/bin/env python3
"""Turn a bench/results/<label>/ directory into the tables of docs/BENCHMARKS.md.   python3 -I bench/summarize.py <dir>"""
import os, statistics, sys

def med(xs):
    return statistics.median(xs)

def read_render(path):
    rounds = {}
    if not os.path.exists(path):
        return rounds
    for line in open(path):
        p = line.split()
        if p and p[0] == "RESULT":
            rounds.setdefault(p[1], []).append(tuple(float(v) for v in p[2:]))
    return rounds

def read_idle(path):
    runs = []
    if not os.path.exists(path):
        return runs
    for line in open(path):
        if not line.startswith("RUN"):
            continue
        d = dict(kv.split("=", 1) for kv in line.split()[2:])
        runs.append(d)
    return runs

def p95(xs):
    xs = sorted(xs)
    return xs[min(len(xs) - 1, int(len(xs) * 0.95))]

def main():
    d = sys.argv[1]
    m, r = read_render(d + "/render-m31.txt"), read_render(d + "/render-ratatui.txt")
    print("| scenario | m31 median us | m31 p95 | ratatui median us | ratatui p95 | m31 / ratatui | m31 bytes | ratatui bytes |")
    print("|---|---:|---:|---:|---:|---:|---:|---:|")
    for name in m:
        mm = med([x[0] for x in m[name]]); mp = med([x[1] for x in m[name]]); mb = m[name][-1][3]
        if name in r:
            rm = med([x[0] for x in r[name]]); rp = med([x[1] for x in r[name]]); rb = r[name][-1][3]
            print("| %s | %.1f | %.1f | %.1f | %.1f | %.2fx | %d | %d |" % (name, mm, mp, rm, rp, mm / rm, mb, rb))
        else:
            print("| %s | %.1f | %.1f | n/a | n/a | | %d | |" % (name, mm, mp, mb))
    print()
    print("(median of the per-round medians, p95 of the per-round p95s; each round is 40 samples of ~10 ms)")
    print()
    print("| app (n runs) | start-up ms med / p95 | RSS KB med | idle CPU ms per s, med / p95 | key to frame ms med / p95 | resize to frame ms med / p95 |")
    print("|---|---:|---:|---:|---:|---:|")
    for label in ("ratatui", "m31-probe-off", "m31-default"):
        runs = read_idle("%s/idle-%s.txt" % (d, label))
        if not runs:
            continue
        st = [float(x["startup_ms"]) for x in runs]
        rss = [int(x["rss_kb"]) for x in runs]
        cpu = [float(x["idle_cpu_ms"]) for x in runs]
        keys = [float(v) for x in runs for v in x["key_ms"].split(",")]
        rez = [float(v) for x in runs for v in x["resize_ms"].split(",")]
        print("| %s (%d) | %.1f / %.1f | %d | %.0f / %.0f | %.1f / %.1f | %.1f / %.1f |" % (label, len(runs), med(st), p95(st), med(rss), med(cpu), p95(cpu), med(keys), p95(keys), med(rez), p95(rez)))

main()
