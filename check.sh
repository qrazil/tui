#!/usr/bin/env bash
# The TUI library's gate.
#
#   M31_ROOT=/path/to/m31 bash check.sh            tests, then the benchmark
#   M31_ROOT=/path/to/m31 bash check.sh --bless    rewrite tests.out from what tests.m31 prints
#
# Four things are checked, and the rule is the repository's own: grep for
# FAILED, never for ok.
#
#   1. every .m31 file here is formatted (`m31c fmt --check`)
#   2. tests.m31, bench.m31, demo.m31 and browse.m31 all compile under gcc
#      and clang, at -O0 and -O2, with no warning from the emitted C
#   3. its output matches tests.out, and contains no `FAIL` line
#   4. the refcount invariant holds: `__rc_live=0` at exit
#
# Then bench.m31 is built and run, and its numbers printed. They are not
# compared against anything -- a time is not a fixture.
#
# This library is one `.m31` file imported by another, compiled by the m31
# compiler (m31c) and then linked, as ordinary C, against the m31 runtime's
# own source files directly -- there is no pre-built runtime library, so
# this script needs both LANGC (the m31c binary) and M31_ROOT (a checkout
# of github.com/qrazil/m31, or an extracted release's bundled runtime SDK,
# containing config.sh and runtime/) -- see build.sh's own header for the
# full reasoning, which this mirrors.
set -uo pipefail
cd "$(dirname "$0")"

if [ -z "${M31_ROOT:-}" ]; then
    echo "M31_ROOT is not set -- point it at a checkout of github.com/qrazil/m31" \
         "(or an extracted release's runtime SDK) matching the m31c version" \
         "you're building with. See build.sh's own header comment." >&2
    exit 1
fi
if [ ! -f "$M31_ROOT/config.sh" ] || [ ! -d "$M31_ROOT/runtime" ]; then
    echo "M31_ROOT=$M31_ROOT does not look like an m31 checkout" \
         "(expected $M31_ROOT/config.sh and $M31_ROOT/runtime/)" >&2
    exit 1
fi

. "$M31_ROOT/config.sh"
. "$M31_ROOT/runtime/arch.sh"

LANGC=${LANGC:-./m31c}
DIR=.
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT

bless=0
[ "${1:-}" = "--bless" ] && bless=1

fail=0
note() { printf '%-34s' "$1"; }
good() { printf '\033[32mok\033[0m\n'; }
bad()  { printf '\033[31mFAILED\033[0m\n'; fail=$((fail + 1)); }

if [ ! -x "$LANGC" ]; then
    echo "compiler not found or not executable: $LANGC" >&2
    exit 1
fi

# 1. formatting -------------------------------------------------------------
note "formatted"
unformatted=""
for f in "$DIR"/*."$LANG_EXT"; do
    out=$("$LANGC" fmt --check "$f" 2>&1) || unformatted="$unformatted$out"$'\n'
done
if [ -n "$unformatted" ]; then bad; echo "$unformatted" | sed 's/^/    /'; else good; fi

# 2. compiles clean, on every compiler present ------------------------------
CCS=()
command -v gcc   >/dev/null && CCS+=("gcc:-O0" "gcc:-O2")
command -v clang >/dev/null && CCS+=("clang:-O0" "clang:-O2")
if [ ${#CCS[@]} -eq 0 ]; then echo "no C compiler" >&2; exit 1; fi

for src in "$DIR"/tests."$LANG_EXT" "$DIR"/bench."$LANG_EXT" "$DIR"/demo."$LANG_EXT" "$DIR"/browse."$LANG_EXT"; do
    base=$(basename "$src" ".$LANG_EXT")
    note "compiles: $base"
    if ! "$LANGC" --emit-c "$src" -o "$WORK/$base.c" 2>"$WORK/$base.diag"; then
        bad; sed 's/^/    /' "$WORK/$base.diag" | head -8; continue
    fi
    bad_cc=0
    for entry in "${CCS[@]}"; do
        cc=${entry%%:*}; opt=${entry##*:}
        # RT_REACTOR_C/RT_CTX_ASM (from runtime/arch.sh above) are paths
        # relative to M31_ROOT, not to this script's own directory.
        if ! "$cc" "$opt" -ffp-contract=off -Wall -Wextra -DRC_DEBUG -I "$M31_ROOT/runtime" \
             -pthread -o "$WORK/$base.$cc$opt" "$WORK/$base.c" \
             "$M31_ROOT/runtime/rt.c" "$M31_ROOT/runtime/scheduler.c" \
             "$M31_ROOT/$RT_REACTOR_C" "$M31_ROOT/$RT_CTX_ASM" \
             2>"$WORK/$base.cc"; then
            bad_cc=1; echo; sed 's/^/    /' "$WORK/$base.cc" | head -8; break
        fi
        if [ -s "$WORK/$base.cc" ]; then
            bad_cc=1; echo; echo "    emitted C produced warnings under $cc $opt"
            sed 's/^/    /' "$WORK/$base.cc" | head -8; break
        fi
    done
    if [ $bad_cc -eq 1 ]; then bad; else good; fi
done

# 3. and 4. the corpus, and the refcount invariant --------------------------
#
# Every build must agree with every other, which is run.sh's layers 1 and 2
# applied to one program: a disagreement between gcc and clang, or between
# -O0 and -O2, means the emitted C leans on something C leaves open.
ref=""; ref_tag=""
for entry in "${CCS[@]}"; do
    cc=${entry%%:*}; opt=${entry##*:}
    got=$("$WORK/tests.$cc$opt" 2>&1)
    note "refcounts: $cc $opt"
    if grep -q '^__rc_live=0$' <<<"$got"; then good; else
        bad; grep '^__rc_live=' <<<"$got" | sed 's/^/    /'
    fi
    got=$(grep -v '^__rc_live=' <<<"$got")
    if [ -z "$ref_tag" ]; then ref=$got; ref_tag="$cc $opt"; else
        note "agrees with $ref_tag"
        if [ "$got" = "$ref" ]; then good; else
            bad; diff <(echo "$ref") <(echo "$got") | head -6 | sed 's/^/    /'
        fi
    fi
done

if [ $bless -eq 1 ]; then
    printf '%s\n' "$ref" > "$DIR/tests.out"
    echo "blessed $DIR/tests.out"
fi

note "no FAIL line"
if grep -q '^FAIL' <<<"$ref"; then
    bad; grep -A2 '^FAIL' <<<"$ref" | head -20 | sed 's/^/    /'
else good; fi

note "output matches tests.out"
if diff -q <(printf '%s\n' "$ref") "$DIR/tests.out" >/dev/null 2>&1; then good; else
    bad; diff "$DIR/tests.out" <(printf '%s\n' "$ref") | head -30 | sed 's/^/    /'
fi

# the benchmark -------------------------------------------------------------
#
# Built again WITHOUT -DRC_DEBUG: that flag makes the runtime count live
# objects, which is exactly the bookkeeping a timing should not include. This
# is the same build `./build.sh` produces.
echo
bcc=gcc
command -v gcc >/dev/null || bcc=clang
if "$bcc" -O2 -ffp-contract=off -I "$M31_ROOT/runtime" -pthread -o "$WORK/bench.fast" \
       "$WORK/bench.c" \
       "$M31_ROOT/runtime/rt.c" "$M31_ROOT/runtime/scheduler.c" \
       "$M31_ROOT/$RT_REACTOR_C" "$M31_ROOT/$RT_CTX_ASM" \
       2>/dev/null; then
    "$WORK/bench.fast"
else
    echo "could not build the benchmark"
fi

echo
if [ $fail -eq 0 ]; then
    printf '\033[32mtui: all checks passed\033[0m\n'
else
    printf '\033[31mtui: %d check(s) FAILED\033[0m\n' "$fail"
fi
exit $fail
