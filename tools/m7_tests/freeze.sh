#!/bin/sh
# Writes data/first_values.golden from the image (test_support::freeze), and with
# `names <addresses file> <out>` names IDA addresses by store and field. Run from the root.
# CARGO_TARGET_DIR is the caller's; the run is under a memory cap and a time limit.
set -e
root=$(pwd)
cap="systemd-run --user --scope -q -p MemoryMax=4G -p MemorySwapMax=0 timeout 2400 nice -n 19"
if [ "$1" = names ]; then
    RC2K_NAME_ADDRESSES=$(realpath "$2") RC2K_ADDRESS_NAMES=$(realpath -m "$3") \
        $cap cargo test --target x86_64-unknown-linux-gnu -p rc2k_decomp --lib -j2 \
        test_support::freeze::name_addresses
else
    RC2K_FREEZE_FIRST_VALUES="$root/data/first_values.golden" \
        RC2K_FIRST_VALUES_MANIFEST="$root/tools/m7_tests/first_values.manifest" \
        $cap cargo test --target x86_64-unknown-linux-gnu -p rc2k_decomp --lib -j2 \
        test_support::freeze::freeze_first_values
fi
