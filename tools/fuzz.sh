#!/bin/bash
# Fuzzes the readers of outside data in rc2k_core: tools/fuzz.sh [seconds a target] [targets].
cd "$(dirname "$0")/.." || exit 1
seconds=${1:-60}
shift
targets=${*:-packed png rle string_table lexicon inbox pcx language}
for t in $targets; do
    echo "== $t"
    cargo +nightly fuzz run "$t" -- -max_total_time="$seconds" -rss_limit_mb=4096 -timeout=20 2>&1 |
        grep -E 'panicked|SUMMARY|Test unit written|Done [0-9]+ runs|^error' | cut -c1-170
done
