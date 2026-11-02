#!/usr/bin/env bash
# The handheld's profile for a profile-guided build (PGO): builds the game instrumented
# (`-C profile-generate`, `--cfg rc2k_pgo` so that it writes its counts as it ends), runs it on
# the handheld through tools/anbernic/pgo_train.py, and merges the counts into
# target/a64-pgo/rc2k.profdata. `./deploy_standalone.sh anbernic pgo` then builds with it.
#
# The profile only steers the compiler's layout and inlining: the program does the same with or
# without it. Functions changed since the training are built as without it, so train again
# after larger changes. Needs llvm-profdata of the same LLVM major version as rustc's
# (`rustc -vV`), and ANBERNIC_HOST in deploy.local (or RC2K_REMOTE_HOST).
set -euo pipefail
cd "$(dirname "$0")/../.."

[ -f deploy.local ] && source deploy.local
HOST="${RC2K_REMOTE_HOST:-${ANBERNIC_HOST:?Set ANBERNIC_HOST in deploy.local}}"
SSH=(ssh -i "${ANBERNIC_KEY:-$HOME/.ssh/rg34xxsp}" -o BatchMode=yes "$HOST")
GAME=/mnt/mmc/Roms/APPS/rally
COUNTS=/tmp/rc2k_pgo

want=$(rustc -vV | sed -n 's/^LLVM version: \([0-9]*\).*/\1/p')
have=$(llvm-profdata --version | sed -n 's/.*LLVM version \([0-9]*\).*/\1/p')
if [ "$want" != "$have" ]; then
    echo "pgo: llvm-profdata is LLVM $have, rustc's is $want" >&2
    exit 1
fi

docker build -q -t rc2k-anbernic tools/anbernic >/dev/null
tools/anbernic/run.sh tools/build_ffmpeg.sh a64
tools/anbernic/run.sh env CARGO_TARGET_DIR=target/a64-pgo \
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C target-cpu=cortex-a53 -C profile-generate=$COUNTS --cfg rc2k_pgo" \
    cargo build --release --target aarch64-unknown-linux-gnu --features remote --bin rc2k

"${SSH[@]}" "pkill -x rc2k_remote; rm -rf $COUNTS; mkdir -p $COUNTS; true"
"${SSH[@]}" "cat > $GAME/rc2k_remote; chmod +x $GAME/rc2k_remote" \
    < target/a64-pgo/aarch64-unknown-linux-gnu/release/rc2k
RC2K_REMOTE_HOST="$HOST" python3 tools/anbernic/pgo_train.py "${1:-85}"

rm -rf target/a64-pgo/counts
mkdir -p target/a64-pgo/counts
scp -q -i "${ANBERNIC_KEY:-$HOME/.ssh/rg34xxsp}" "$HOST:$COUNTS/*.profraw" target/a64-pgo/counts/
"${SSH[@]}" "rm -rf $COUNTS"
llvm-profdata merge -o target/a64-pgo/rc2k.profdata target/a64-pgo/counts/*.profraw
echo "pgo: target/a64-pgo/rc2k.profdata ($(du -h target/a64-pgo/rc2k.profdata | cut -f1))"
