#!/bin/bash
# Fuzzes the readers of outside data in rc2k_core: tools/fuzz.sh [seconds a target] [targets].
# Every target runs; the exit status is cargo's: nonzero when any target failed.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 1
seconds=${1:-60}
shift || true
targets=${*:-packed png rle string_table lexicon inbox pcx language mpeg1_video mp2 indeo4 net_wire tailscale}
status=0
for t in $targets; do
    echo "== $t"
    # The summary lines are shown; pipefail keeps cargo's failure as the pipeline's status
    # (grep finding no line is not a failure here).
    cargo +nightly fuzz run "$t" -- -max_total_time="$seconds" -rss_limit_mb=4096 -timeout=20 2>&1 |
        { grep -E 'panicked|SUMMARY|Test unit written|Done [0-9]+ runs|^error' || true; } | cut -c1-170
    [ "${PIPESTATUS[0]}" -eq 0 ] || status=1
done
exit "$status"
