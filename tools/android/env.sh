# Source this: the Android tools' places (everything lives in $RC2K_ANDROID_HOME, nothing system-wide).
export RC2K_ANDROID_HOME=${RC2K_ANDROID_HOME:-/mnt/discoIII/android}
export ANDROID_HOME=$RC2K_ANDROID_HOME/sdk
export ANDROID_SDK_ROOT=$ANDROID_HOME
export NDK_VERSION=${NDK_VERSION:-28.2.13676358}
export ANDROID_NDK_HOME=$ANDROID_HOME/ndk/$NDK_VERSION
export NDK_BIN=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin
export API=${API:-26}
export PATH=$ANDROID_HOME/platform-tools:$ANDROID_HOME/cmdline-tools/latest/bin:$PATH
# Cargo's cross settings for one ABI: android_cargo_env x86_64|arm64
android_cargo_env() {
    local triple abi
    case "$1" in
        x86_64) triple=x86_64-linux-android; abi=x86_64 ;;
        arm64) triple=aarch64-linux-android; abi=arm64-v8a ;;
        *) echo "unknown abi $1" >&2; return 1 ;;
    esac
    local upper=${triple^^}; upper=${upper//-/_}
    export CARGO_BUILD_TARGET=$triple
    export "CARGO_TARGET_${upper}_LINKER=$NDK_BIN/${triple}${API}-clang"
    local cc=$NDK_BIN/${triple}${API}-clang
    export "CC_${triple//-/_}=$cc" "CXX_${triple//-/_}=${cc}++" "AR_${triple//-/_}=$NDK_BIN/llvm-ar"
    export RC2K_ANDROID_SDL_LIB=$RC2K_ANDROID_HOME/sdl3build/$abi
    export ANDROID_ABI=$abi
}

# The source of the `sdl3-src` crate the Rust build uses (the version is the lockfile's, the one
# place it is named): sdl_source_dir prints its `SDL` folder; the crate must have been fetched.
sdl_source_dir() {
    local lock version dir
    lock="$(dirname "${BASH_SOURCE[0]}")/../../Cargo.lock"
    version=$(grep -A1 '^name = "sdl3-src"$' "$lock" | sed -n 's/^version = "\(.*\)"$/\1/p')
    [ -n "$version" ] || { echo "no sdl3-src in $lock" >&2; return 1; }
    for dir in "$HOME"/.cargo/registry/src/*/"sdl3-src-$version"/SDL; do
        [ -d "$dir" ] && { echo "$dir"; return 0; }
    done
    echo "sdl3-src-$version is not fetched (cargo fetch)" >&2
    return 1
}
