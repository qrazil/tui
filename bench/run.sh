#!/usr/bin/env bash
# Builds the m31 and ratatui benchmarks and runs them alternately, pinned to one core.
#
#   M31_ROOT=/path/to/m31 LANGC=/path/to/m31c bash bench/run.sh [rounds] [idle_runs]
#
# Needs: the m31 toolchain (as tests/test.sh), cargo, python3, taskset (Linux). Environment:
#   BENCH_CPU   core to pin to (default 3)        BENCH_OUT  results directory (default bench/results/<label>)
#   BENCH_LABEL a name for this run (default the m31c file's version-less name, e.g. m31rel9)
#   CARGO_TARGET_DIR  where ratatui is built (default a temp directory; nothing under bench/ is built in place)
# Writes <out>/render-m31.txt, render-ratatui.txt (RESULT lines, one block per round) and idle-*.txt (RUN lines).
set -uo pipefail
cd "$(dirname "$0")/.."
rounds=${1:-3}
idle_runs=${2:-30}
cpu=${BENCH_CPU:-3}
: "${M31_ROOT:?set M31_ROOT}" "${LANGC:?set LANGC}"
label=${BENCH_LABEL:-$(basename "$M31_ROOT")}
out=${BENCH_OUT:-bench/results/$label}
mkdir -p "$out"
work=$(mktemp -d); trap 'rm -rf "$work"' EXIT
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$work/cargo}

bash scripts/build.sh bench/bench.m31 -o "$work/bench" >/dev/null 2>&1 || { echo "m31 bench build failed" >&2; exit 1; }
bash scripts/build.sh bench/idle.m31 -o "$work/idle" >/dev/null 2>&1 || { echo "m31 idle build failed" >&2; exit 1; }
(cd bench/ratatui && cargo build --release --locked >/dev/null 2>&1) || { echo "ratatui build failed" >&2; exit 1; }
rbin=$CARGO_TARGET_DIR/release

{ echo "machine: $(grep -m1 'model name' /proc/cpuinfo | cut -d: -f2 | sed 's/^ //'), $(nproc) threads, governor $(cat /sys/devices/system/cpu/cpu$cpu/cpufreq/scaling_governor 2>/dev/null)"
  echo "kernel: $(uname -sr), pinned to cpu $cpu, loadavg: $(cut -d' ' -f1-3 /proc/loadavg)"
  echo "m31: $(grep -m1 '^Tag:' "$M31_ROOT/NOTES.txt" 2>/dev/null || "$LANGC" --version 2>&1 | head -1)  rustc: $(rustc --version)  cc: $(cc --version | head -1)"; } > "$out/machine.txt"
: > "$out/render-m31.txt"; : > "$out/render-ratatui.txt"
for r in $(seq 1 "$rounds"); do
    echo "ROUND $r" >> "$out/render-ratatui.txt"; taskset -c "$cpu" "$rbin/tui-bench-ratatui" >> "$out/render-ratatui.txt"
    echo "ROUND $r" >> "$out/render-m31.txt";     taskset -c "$cpu" "$work/bench" >> "$out/render-m31.txt"
done
taskset -c "$cpu" python3 -I bench/idle.py ratatui "$idle_runs" "$rbin/idle" > "$out/idle-ratatui.txt"
taskset -c "$cpu" python3 -I bench/idle.py m31-probe-off "$idle_runs" "$work/idle" TUI_WIDTH_PROBE=off > "$out/idle-m31-probe-off.txt"
taskset -c "$cpu" python3 -I bench/idle.py m31-default "$idle_runs" "$work/idle" > "$out/idle-m31-default.txt"
python3 -I bench/summarize.py "$out" | tee "$out/summary.md"
