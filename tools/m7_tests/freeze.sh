#!/bin/sh
# Writes data/first_values.golden from the stores' first values (test_support::freeze) and
# tools/m7_tests/first_values.manifest. Run from the root. CARGO_TARGET_DIR is the caller's;
# the run is under a memory cap and a time limit.
set -e
root=$(pwd)
cap="systemd-run --user --scope -q -p MemoryMax=4G -p MemorySwapMax=0 timeout 2400 nice -n 19"
RC2K_FREEZE_FIRST_VALUES="$root/data/first_values.golden" \
    RC2K_FIRST_VALUES_MANIFEST="$root/tools/m7_tests/first_values.manifest" \
    $cap cargo test --target x86_64-unknown-linux-gnu -p rc2k_decomp --lib -j2 \
    test_support::freeze::freeze_first_values
