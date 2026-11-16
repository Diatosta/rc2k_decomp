#!/usr/bin/env bash
# build.sh: builds the program for one target into dist/<target>/, and with --deploy copies
# that folder where deploy.local says. Run on Linux or macOS; on Windows itself use build.ps1.
#
#     ./build.sh <target> [--deploy] [--release|--debug] [--pgo] [--arch x64|a64]
#     ./build.sh                    lists the targets
#
# dist/<target>/ holds the program and everything it needs beside it, the same for every
# target: copy the folder's contents into the game's folder (the one with `files` and `var`)
# and it runs. Cargo's own folders stay as they were (target/, target/x64, target/linux ..).
#
# deploy.local (not in git; shell variables) names where --deploy copies each target:
#     DEPLOY_WINDOWS=<folder>        windows
#     DEPLOY_WINDOWS_X64=<folder>    windows-x64
#     DEPLOY_LINUX=<folder>          linux (default: ~/.local/share/rally-championship)
#     DEPLOY_WEB=<folder>            web
#     ANBERNIC_HOST=root@address     handheld, over SSH (ANBERNIC_KEY, ANBERNIC_GAME)
#     MAC_HOST=user@address          mac, built on that Mac over SSH when this is not one
#                                    (MAC_KEY, MAC_PORT, MAC_SRC); MAC_GAME_DIR its game folder
# RC2K_FEATURES in the environment adds cargo features (e.g. RC2K_FEATURES=remote).
set -euo pipefail

cd "$(dirname "$0")"

usage() {
    cat <<'EOF'
usage: ./build.sh <target> [--deploy] [--release|--debug]

targets:
  windows       Windows, 32 bits (crossed with cargo-xwin)    dist/windows/
  windows-x64   Windows, 64 bits (crossed with cargo-xwin)    dist/windows-x64/
  linux         Linux, x86-64                                 dist/linux/
  mac           macOS, Intel and Apple silicon in one bundle  dist/mac/
                (on a Mac; from elsewhere on MAC_HOST over SSH; --arch x64|a64 for one)
  handheld      Anbernic RG34XXSP (in its Docker container)   dist/handheld/
                (--pgo: with the profile of tools/anbernic/pgo.sh)
  web           WebAssembly through Emscripten (experimental) dist/web/

--deploy   also copy dist/<target>/ where deploy.local says (see the top of this script)
--debug    an unoptimised build (default: --release)
EOF
}

fail() { echo "build.sh: $*" >&2; exit 1; }

# Stops with what to install when a tool is not there.
need() {
    command -v "$1" >/dev/null 2>&1 || fail "no $1: $2"
}

TARGET=""
DEPLOY=false
PROFILE=release
PGO=false
MAC_ARCH=""
while [ $# -gt 0 ]; do
    case "$1" in
        --deploy) DEPLOY=true ;;
        --release) PROFILE=release ;;
        --debug) PROFILE=debug ;;
        --pgo) PGO=true ;;
        --arch) shift; MAC_ARCH="${1:-}" ;;
        -h|--help) usage; exit 0 ;;
        -*) usage >&2; fail "unknown switch $1" ;;
        *) [ -z "$TARGET" ] || fail "one target at a time (got $TARGET and $1)"; TARGET="$1" ;;
    esac
    shift
done
if [ -z "$TARGET" ]; then usage; exit 0; fi

# shellcheck disable=SC1091
[ -f deploy.local ] && source deploy.local

DIST="dist/$TARGET"
# Cargo's switches for the profile and the features, and the profile's folder name.
CARGO_FLAGS=()
[ "$PROFILE" = release ] && CARGO_FLAGS+=(--release)
[ -n "${RC2K_FEATURES:-}" ] && CARGO_FLAGS+=(--features "$RC2K_FEATURES")

# An empty dist/<target>/, with the own texts of every language in locale/ (the installer copies
# the installed language's into the game's files/locale).
fresh_dist() {
    rm -rf "$DIST"
    mkdir -p "$DIST/locale"
    cp assets/locale/*.rc2k.toml "$DIST/locale/"
}

# The own texts of the language installed in a game folder, from dist/<target>/locale/ into its
# files/locale (named after the language's .LOC file, whatever its case).
deploy_own_texts() {
    local dest="$1" loc stem
    loc=$(find "$dest/files" -maxdepth 2 -ipath '*locale/*.loc' 2>/dev/null | sort | head -1)
    [ -n "$loc" ] || return 0
    stem=$(basename "$loc"); stem=${stem%.*}
    local from
    from=$(find "$DIST/locale" -maxdepth 1 -iname "$stem.rc2k.toml" | head -1)
    if [ -n "$from" ]; then
        cp "$from" "$(dirname "$loc")/$(basename "$from")"
        echo "own texts: $(basename "$from") into $(dirname "$loc")"
    else
        echo "own texts: none for the language $stem (the game shows English)"
    fi
}

# Copies dist/<target>/ into a folder, replacing what is there by the same names (a program
# that is running keeps its old file).
deploy_to_folder() {
    local dest="$1" item
    mkdir -p "$dest"
    for item in "$DIST"/*; do
        # The languages' own texts are the installer's to pick from: only the installed one goes
        # in (deploy_own_texts), not the whole folder.
        [ "$(basename "$item")" = locale ] && continue
        rm -rf "$dest/$(basename "$item")"
        cp -R "$item" "$dest/"
    done
    deploy_own_texts "$dest"
    echo "deployed $(ls "$DIST" | tr '\n' ' ')to $dest"
}

# With --deploy: stops before the build when deploy.local lacks the target's line.
need_deploy_line() {
    local name="$1" example="$2"
    if $DEPLOY && [ -z "${!name:-}" ]; then
        fail "--deploy for $TARGET needs a line $name=$example in deploy.local"
    fi
}

report() {
    echo "built $DIST/:"
    (cd "$DIST" && find . -type f -exec ls -l {} + | awk '{printf "  %10d  %s\n", $5, $NF}')
}

build_windows() {
    local arch="$1" triple="$2" xwin_arch="$3" target_dir="$4"
    need cargo "install rustup (https://rustup.rs)"
    cargo xwin --version >/dev/null 2>&1 || fail "no cargo-xwin: cargo install --locked cargo-xwin"
    need clang-cl "the Windows builds need LLVM's clang-cl, lld-link, llvm-lib and llvm-nm (Clang 19 or later)"
    need lld-link "the Windows builds need LLVM's lld-link"
    need curl "SDL's library is downloaded with curl"
    need unzip "SDL's library comes as a zip"
    tools/fetch_sdl3.sh
    CARGO_TARGET_DIR="$target_dir" cargo xwin build ${CARGO_FLAGS[@]+"${CARGO_FLAGS[@]}"} \
        --xwin-arch "$xwin_arch" --target "$triple" --bin rc2k
    fresh_dist
    cp "$target_dir/$triple/$PROFILE/rc2k.exe" "third_party/sdl3/lib/$arch/SDL3.dll" "$DIST/"
}

build_linux() {
    need cargo "install rustup (https://rustup.rs)"
    need cmake "SDL is built from source: install cmake and your system's video and sound development packages"
    CARGO_TARGET_DIR=target/linux cargo build ${CARGO_FLAGS[@]+"${CARGO_FLAGS[@]}"} \
        --target x86_64-unknown-linux-gnu --bin rc2k
    fresh_dist
    cp "target/linux/x86_64-unknown-linux-gnu/$PROFILE/rc2k" "$DIST/"
    if command -v objdump >/dev/null 2>&1; then
        echo "the Linux program needs glibc $(objdump -T "$DIST/rc2k" | grep -v ' w ' \
            | grep -oE 'GLIBC_[0-9.]+' | sort -uV | tail -1 | cut -d_ -f2) or newer"
    fi
}

# The handheld: built in a container of its stock system, the program stripped (the one with
# its line tables stays in cargo's folder, for the profiler's addresses).
build_handheld() {
    need docker "the handheld's build runs in a container (tools/anbernic/Dockerfile)"
    docker build -q -t rc2k-anbernic tools/anbernic >/dev/null
    local built
    if $PGO; then
        local profile="$PWD/target/a64-pgo/rc2k.profdata"
        [ -f "$profile" ] || fail "no $profile: run tools/anbernic/pgo.sh first"
        tools/anbernic/run.sh env CARGO_TARGET_DIR=target/a64-linux-pgo \
            CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-C target-cpu=cortex-a53 -C profile-use=$profile -C llvm-args=-pgo-warn-missing-function=false" \
            cargo build ${CARGO_FLAGS[@]+"${CARGO_FLAGS[@]}"} --target aarch64-unknown-linux-gnu --bin rc2k
        built="target/a64-linux-pgo/aarch64-unknown-linux-gnu/$PROFILE/rc2k"
    else
        tools/anbernic/run.sh cargo build ${CARGO_FLAGS[@]+"${CARGO_FLAGS[@]}"} \
            --target aarch64-unknown-linux-gnu --bin rc2k
        built="target/a64-linux/aarch64-unknown-linux-gnu/$PROFILE/rc2k"
    fi
    fresh_dist
    tools/anbernic/run.sh aarch64-linux-gnu-strip -o "$DIST/rc2k" "$built"
    cp tools/anbernic/Rally.sh "$DIST/"
    cp assets/handheld.png "$DIST/rally-image.png"
    chmod +x "$DIST/rc2k" "$DIST/Rally.sh"
}

# Over SSH into the device's ports folder: the program in the game's folder, and beside that
# folder the entry the device's menu lists.
deploy_handheld() {
    local key="${ANBERNIC_KEY:-$HOME/.ssh/rg34xxsp}"
    [ -f "$key" ] || fail "no SSH key $key: set ANBERNIC_KEY=<ssh key> in deploy.local"
    local ssh=(ssh -i "$key" -o BatchMode=yes "$ANBERNIC_HOST")
    local game="${ANBERNIC_GAME:-/userdata/roms/ports/rally}"
    "${ssh[@]}" "rm -f $game/rc2k"
    "${ssh[@]}" "cat > $game/rc2k; chmod +x $game/rc2k" < "$DIST/rc2k"
    "${ssh[@]}" "cat > $game/../Rally.sh; chmod +x $game/../Rally.sh" < "$DIST/Rally.sh"
    # The entry's picture in the device's menu: the icon in the ports' images folder, named in
    # the ports' game list (the line added once; the list made if there is none). The menu
    # (EmulationStation, through its local API; nothing if it is not up) is told to read its
    # lists before and after the edit: a reload first saves what it holds (play counts), which
    # would otherwise be written over the edit.
    "${ssh[@]}" "mkdir -p $game/../images; cat > $game/../images/rally-image.png" < "$DIST/rally-image.png"
    "${ssh[@]}" "cd $game/.. && curl -s -m 10 http://127.0.0.1:1234/reloadgames >/dev/null 2>&1; sleep 2
      if [ ! -f gamelist.xml ]; then
        printf '<?xml version=\"1.0\"?>\n<gameList>\n\t<game>\n\t\t<path>./Rally.sh</path>\n\t\t<name>Rally</name>\n\t</game>\n</gameList>\n' > gamelist.xml; fi
      if ! grep -q 'rally-image.png' gamelist.xml; then
        cp gamelist.xml gamelist.xml.before_rally_image
        if grep -q '<path>./Rally.sh</path>' gamelist.xml; then
          sed -i 's|<path>./Rally.sh</path>|&\n\t\t<image>./images/rally-image.png</image>|' gamelist.xml
        else
          sed -i 's|</gameList>|\t<game>\n\t\t<path>./Rally.sh</path>\n\t\t<name>Rally</name>\n\t\t<image>./images/rally-image.png</image>\n\t</game>\n</gameList>|' gamelist.xml
        fi
      fi
      curl -s -m 5 http://127.0.0.1:1234/reloadgames >/dev/null 2>&1 || true"
    echo "deployed rc2k, Rally.sh and the menu's picture to the handheld ($game)"
}

# macOS, on a Mac: each processor's program (the other one crossed), joined by lipo into
# `Rally Championship.app` and signed ad hoc (not notarised). SDL is linked in.
build_mac() {
    export PATH="$HOME/.cargo/bin:/opt/local/bin:/opt/homebrew/bin:/usr/local/bin:$PATH"
    export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-11.0}"
    need cargo "install rustup (https://rustup.rs) with the two apple-darwin targets"
    need cmake "SDL is built from source: install cmake (MacPorts or Homebrew)"
    need lipo "install the Command Line Tools (xcode-select --install)"
    local triples=() bins=() triple
    case "$MAC_ARCH" in
        x64) triples=(x86_64-apple-darwin) ;;
        a64) triples=(aarch64-apple-darwin) ;;
        "") triples=(x86_64-apple-darwin aarch64-apple-darwin) ;;
        *) fail "--arch takes x64 or a64" ;;
    esac
    for triple in "${triples[@]}"; do
        CARGO_TARGET_DIR=target/mac cargo build ${CARGO_FLAGS[@]+"${CARGO_FLAGS[@]}"} \
            --target "$triple" --bin rc2k
        bins+=("target/mac/$triple/$PROFILE/rc2k")
    done
    fresh_dist
    local app="$DIST/Rally Championship.app"
    mkdir -p "$app/Contents/MacOS"
    lipo -create -output "$app/Contents/MacOS/rc2k" "${bins[@]}"
    cp tools/mac/Info.plist "$app/Contents/"
    mkdir -p "$app/Contents/Resources"
    cp assets/icon.icns "$app/Contents/Resources/"
    codesign --force --sign - "$app"
    echo "Rally Championship.app holds: $(lipo -archs "$app/Contents/MacOS/rc2k")"
}

# macOS from another system: the source copied to the Mac that deploy.local names and built
# there by this script; the bundle comes back into dist/mac/ here.
build_mac_over_ssh() {
    [ -n "${MAC_HOST:-}" ] || fail "the mac target is built on a Mac: run this there, or set MAC_HOST=<user@address> in deploy.local to build on one over SSH (crossing from Linux by hand: tools/mac/README.md)"
    need rsync "the source is copied to the Mac with rsync"
    local key="${MAC_KEY:-$HOME/.ssh/rc2k_mac}"
    [ -f "$key" ] || fail "no SSH key $key: set MAC_KEY in deploy.local"
    local ssh="ssh -i $key -p ${MAC_PORT:-22} -o BatchMode=yes"
    local src="${MAC_SRC:-rc2k_src}" there="mac --$PROFILE"
    [ -n "$MAC_ARCH" ] && there="$there --arch $MAC_ARCH"
    $DEPLOY && there="$there --deploy"
    rsync -a --delete -e "$ssh" --exclude=/target/ --exclude=/dist/ --exclude=/ida_db/ \
        --exclude=/.git/ --exclude=/third_party --exclude=/fuzz/target/ --exclude=/deploy.local \
        ./ "$MAC_HOST:$src/"
    $ssh "$MAC_HOST" "cd $src && MAC_GAME_DIR='${MAC_GAME_DIR:-}' RC2K_FEATURES='${RC2K_FEATURES:-}' ./build.sh $there"
    fresh_dist
    rsync -a -e "$ssh" "$MAC_HOST:$src/dist/mac/" "$DIST/"
}

build_web() {
    [ -f "$HOME/emsdk/emsdk_env.sh" ] || fail "no Emscripten SDK in ~/emsdk (https://emscripten.org/docs/getting_started/downloads.html)"
    rustup toolchain list | grep -q '^nightly' || fail "no nightly toolchain: rustup toolchain install nightly --target wasm32-unknown-emscripten --component rust-src"
    local web_flags=()
    [ "$PROFILE" = debug ] && web_flags+=(--debug)
    [ -n "${RC2K_FEATURES:-}" ] && web_flags+=(--features "$RC2K_FEATURES")
    tools/web/build.sh ${web_flags[@]+"${web_flags[@]}"}
    fresh_dist
    local out="target/web/wasm32-unknown-emscripten/$PROFILE"
    cp "$out/rc2k.js" "$out/rc2k.wasm" tools/web/index.html tools/web/unzip.js tools/web/serve.py "$DIST/"
    echo "serve it with: python3 $DIST/serve.py $DIST"
}

case "$TARGET" in
    windows)
        need_deploy_line DEPLOY_WINDOWS "<the game's folder>"
        build_windows x86 i686-pc-windows-msvc x86 target
        report
        if $DEPLOY; then deploy_to_folder "$DEPLOY_WINDOWS"; fi
        ;;
    windows-x64)
        need_deploy_line DEPLOY_WINDOWS_X64 "<folder, e.g. x64 in the game's folder>"
        build_windows x64 x86_64-pc-windows-msvc x86_64 target/x64
        report
        if $DEPLOY; then deploy_to_folder "$DEPLOY_WINDOWS_X64"; fi
        ;;
    linux)
        build_linux
        report
        if $DEPLOY; then deploy_to_folder "${DEPLOY_LINUX:-$HOME/.local/share/rally-championship}"; fi
        ;;
    handheld)
        need_deploy_line ANBERNIC_HOST "root@<address>"
        build_handheld
        report
        if $DEPLOY; then deploy_handheld; fi
        ;;
    mac)
        if [ "$(uname)" = Darwin ]; then
            build_mac
            report
            if $DEPLOY; then deploy_to_folder "${MAC_GAME_DIR:-$HOME/rally}"; fi
        else
            build_mac_over_ssh
            report
        fi
        ;;
    web)
        need_deploy_line DEPLOY_WEB "<folder>"
        build_web
        report
        if $DEPLOY; then deploy_to_folder "$DEPLOY_WEB"; fi
        ;;
    *)
        usage >&2
        fail "unknown target $TARGET"
        ;;
esac
