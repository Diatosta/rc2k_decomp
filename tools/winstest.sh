#!/bin/bash
# tools/winstest.sh: the Windows installer end to end under Wine in a throwaway prefix (the
# x64 build with `remote`), as tools/instest.sh is for Linux: every page accepted with Return;
# with EDIT_AT=n, the folder text edited on page n first (EDIT_KEYS, SDL's scancode names;
# by default two Backspaces, X and Backslash). INSTALL_OK when the record is written and
# every file of the download folder's FILES is installed. Never passes -u, -i or -p.
# Wine runs in its own Xvfb display when Xvfb is installed (WINSTEST_DESKTOP=1: on the
# desktop's), so a locked or busy desktop does not stop it.
#
#   RC2K_DOWNLOAD=<folder with the CD's files> [EDIT_AT=3] timeout 1500 bash tools/winstest.sh [nobuild]
REPO=$(cd "$(dirname "$0")/.." && pwd)
SRC=${RC2K_DOWNLOAD:?set RC2K_DOWNLOAD to the folder holding the files of the CD}
BOX=$HOME/.cache/rc2k_winst; RH=$BOX/remote; DL=$BOX/download
export WINEPREFIX=$BOX/prefix WINEDEBUG=-all
export RC2K_REMOTE_HOME=$RH
R="python3 $REPO/tools/remote.py"
XVFB=
stop_x() { [ -n "$XVFB" ] && kill $XVFB 2>/dev/null; }
fail() { echo "INSTALL_FAIL: $*"; grep -h "refused\|ending" $DL/rc2k.log 2>/dev/null | tail -3; wineserver -k 2>/dev/null; stop_x; exit 1; }
[ "$1" = nobuild ] || { cd $REPO && CARGO_TARGET_DIR=target/x64-remote timeout 1200 nice -n 10 cargo xwin build --release \
   --xwin-arch x86_64 --target x86_64-pc-windows-msvc --bin rc2k --features remote > $BOX.build.log 2>&1 || fail "build"; }
EXE=$REPO/target/x64-remote/x86_64-pc-windows-msvc/release/rc2k.exe
[ -f $EXE ] || fail "no exe"
wineserver -k 2>/dev/null; rm -rf $BOX; mkdir -p $RH/ctl $DL
if [ "${WINSTEST_DESKTOP:-0}" != 1 ] && command -v Xvfb >/dev/null; then
  for n in $(seq 90 99); do [ -e /tmp/.X11-unix/X$n ] || [ -e /tmp/.X$n-lock ] || break; done
  Xvfb :$n -screen 0 1280x1024x24 -nolisten tcp > $BOX.xvfb.log 2>&1 &
  XVFB=$!; sleep 2; kill -0 $XVFB 2>/dev/null || fail "Xvfb :$n did not start"
  export DISPLAY=:$n; unset WAYLAND_DISPLAY
fi
for f in "$SRC"/*; do
  case "$(basename "$f")" in rc2k|rc2k.exe|rc2k.log|var|savedata|files|linux|x64) ;; *) ln -s "$f" $DL/;; esac
done
cp $EXE $DL/; cp $REPO/third_party/sdl3/lib/x64/SDL3.dll $DL/
timeout 120 wineboot -i >/dev/null 2>&1
export SDL_AUDIODRIVER=dummy SDL_VIDEODRIVER=offscreen RC2K_REMOTE="Z:${RH//\//\\}\\ctl" \
  SDL_JOYSTICK_IGNORE_DEVICES_EXCEPT=0x0000/0x0000 SDL_GAMECONTROLLER_IGNORE_DEVICES_EXCEPT=0x0000/0x0000
cd $DL; nohup wine rc2k.exe > $RH/rc2k.out 2>&1 < /dev/null &
printf %s $! > $RH/pid
sleep 3
for i in $(seq 40); do timeout 5 $R state >/dev/null 2>&1 && break; sleep 1; done
timeout 5 $R state >/dev/null || fail "game does not answer: $(tail -3 $RH/rc2k.out)"
rec() { find $WINEPREFIX/drive_c -iname install.cfg -path '*var*' 2>/dev/null | head -1; }
for n in $(seq 1 12); do
  [ -n "$(rec)" ] && break
  # The installer ends once the installed game has started: a quick install may be done between pages.
  sleep 1.5; [ -n "$(rec)" ] && break
  timeout 20 $R shot page$n >/dev/null || { sleep 2; [ -n "$(rec)" ] && break; fail "shot $n (the game ended?)"; }
  if [ "${EDIT_AT:-0}" = $n ]; then
    i=0; for k in ${EDIT_KEYS:-Backspace Backspace X Backslash}; do i=$((i+1)); timeout 10 $R key $k 120 >/dev/null || fail "key $k"; sleep 0.5; timeout 20 $R shot edit$i >/dev/null || fail "after key $i ($k)"; done
    timeout 20 $R shot page${n}edit >/dev/null
  fi
  timeout 10 $R key Return 120 >/dev/null || { sleep 2; [ -n "$(rec)" ] && break; fail "Return on page $n"; }
done
for i in $(seq 120); do [ -n "$(rec)" ] && break; timeout 5 $R state >/dev/null 2>&1 || break; sleep 2; done
[ -n "$(rec)" ] || fail "no install record"
echo "record: $(rec)"; grep -h "refused\|ending" $DL/rc2k.log | tail -2
INST=$(dirname "$(dirname "$(rec)")")
(cd $SRC/FILES && find . -type f | tr a-z A-Z | sort -u) > $BOX/src.lst
(cd "$INST/files" 2>/dev/null && find . -type f | tr a-z A-Z | sort -u) > $BOX/dst.lst
MISSING=$(comm -23 $BOX/src.lst $BOX/dst.lst | wc -l)
echo "installed in $INST: source files $(wc -l < $BOX/src.lst), installed $(wc -l < $BOX/dst.lst), missing $MISSING"
[ "$MISSING" = 0 ] || fail "$MISSING files missing"
echo INSTALL_OK
wineserver -k 2>/dev/null; stop_x
