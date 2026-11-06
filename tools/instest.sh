#!/bin/bash
# Sandboxed installer end-to-end test (Linux): the installer run from a throwaway copy of the
# download folder, into a throwaway home, every page accepted with Return; INSTALL_OK when
# every file of the download folder's FILES is installed and the install record written.
# The real home and install are never touched. Never passes -u, -i or -p.
#
#   RC2K_DOWNLOAD=<folder with the CD's files> timeout 900 bash tools/instest.sh [nobuild]
#
# BIN=<binary> tests another build; screenshots of each page in ~/.cache/rc2k_instest/remote/shots.
REPO=$(cd "$(dirname "$0")/.." && pwd)
SRC=${RC2K_DOWNLOAD:?set RC2K_DOWNLOAD to the folder holding the files of the CD}
BOX=$HOME/.cache/rc2k_instest
RH=$BOX/remote            # RC2K_REMOTE_HOME for remote.py
DL=$BOX/download          # throwaway download folder (symlinks to the CD's files)
SH=$BOX/home              # sandbox HOME / XDG_DATA_HOME
export RC2K_REMOTE_HOME=$RH
R="python3 $REPO/tools/remote.py"
PID=
fail() { echo "INSTALL_FAIL: $*"; cleanup; exit 1; }
cleanup() {
  if [ -n "$PID" ] && kill -0 "$PID" 2>/dev/null && [ "$(cat /proc/$PID/comm 2>/dev/null)" = rc2k ]; then
    timeout 10 $R stop >/dev/null 2>&1; sleep 1
    kill -0 "$PID" 2>/dev/null && kill "$PID" 2>/dev/null; sleep 1
    kill -0 "$PID" 2>/dev/null && kill -9 "$PID" 2>/dev/null
  fi
}
trap cleanup EXIT
[ "$1" = nobuild ] || {
  cd $REPO && systemd-run --user --scope --quiet -p MemoryMax=4G -p MemorySwapMax=0 \
    env CARGO_TARGET_DIR=target/linux-remote timeout 900 nice -n 10 cargo build --release \
    --target x86_64-unknown-linux-gnu --features remote --bin rc2k >$BOX.build.log 2>&1 \
    || fail "build (see $BOX.build.log)"; }
BIN=${BIN:-$REPO/target/linux-remote/x86_64-unknown-linux-gnu/release/rc2k}
[ -x $BIN ] || fail "no binary"
rm -rf $BOX; mkdir -p $RH/ctl $DL $SH/.local/share
# the download folder: everything of the CD linked, except the stale binary/logs/var
for f in "$SRC"/*; do
  case "$(basename "$f")" in rc2k|rc2k.log|var|savedata|files) ;; *) ln -s "$f" $DL/;; esac
done
cp $BIN $DL/rc2k
INSTALLDIR=$SH/.local/share/rally-championship
export HOME=$SH XDG_DATA_HOME=$SH/.local/share XDG_CONFIG_HOME=$SH/.config
export SDL_AUDIODRIVER=dummy SDL_VIDEODRIVER=offscreen RC2K_REMOTE=$RH/ctl
mkdir -p $SH/.config
cd $DL
nohup ./rc2k >$RH/rc2k.out 2>&1 </dev/null &
PID=$!; printf %s $PID >$RH/pid
sleep 1
kill -0 $PID 2>/dev/null || fail "binary exited at start: $(tail -3 $RH/rc2k.out)"
cd $BOX
# wait for the first frame (state answers)
for i in $(seq 30); do timeout 5 $R state >/dev/null 2>&1 && break; sleep 1; done
timeout 5 $R state || fail "game does not answer"
step() { # name keys...
  local n=$1; shift
  sleep 1.5
  timeout 20 $R shot $n >/dev/null || fail "shot $n"
  for k in "$@"; do timeout 10 $R key $k 120 >/dev/null || fail "key $k"; sleep 0.4; done
}
# The pages (src/install): main page ("install" highlighted), then Return accepts each page
# (language, folder = the default $XDG_DATA_HOME/rally-championship, marks, finish).
# Generic driver: shot, Return, until the game ends or the install record appears.
alive() { kill -0 $PID 2>/dev/null; }
for n in $(seq 1 12); do
  alive || break
  [ -f $INSTALLDIR/var/install.cfg ] && break
  step page$n Return
  if grep -q "install refused\|not ported\|fatal" $DL/rc2k.log 2>/dev/null && ! alive; then break; fi
done
# the copy: wait up to 240 s for the record (written last) or the end of the game
for i in $(seq 120); do
  [ -f $INSTALLDIR/var/install.cfg ] && break
  alive || break
  sleep 2
done
timeout 20 $R shot final >/dev/null 2>&1
sleep 3
if ! [ -f $INSTALLDIR/var/install.cfg ]; then
  fail "no var/install.cfg in $INSTALLDIR; log: $(grep -h 'refused\|ending' $DL/rc2k.log | tail -2 | tr '\n' ' ')"
fi
# compare the file lists: the CD's FILES tree against the installed files tree
(cd $SRC/FILES && find . -type f | tr a-z A-Z | sort -u) >$BOX/src.lst
(cd $INSTALLDIR/files 2>/dev/null && find . -type f | tr a-z A-Z | sort -u) >$BOX/dst.lst
[ -s $BOX/src.lst ] && [ -s $BOX/dst.lst ] || fail "an empty file list (source or install)"
MISSING=$(comm -23 $BOX/src.lst $BOX/dst.lst | wc -l)
echo "source files $(wc -l <$BOX/src.lst), installed $(wc -l <$BOX/dst.lst), missing $MISSING; shots in $RH/shots"
[ "$MISSING" = 0 ] || fail "$MISSING files missing (see $BOX/src.lst, dst.lst)"
echo INSTALL_OK
exit 0
