# source tools/mac/cross/env.sh [x64|a64]: the environment for `cargo build --target
# x86_64-apple-darwin` (or aarch64-apple-darwin) on Linux, as tools/mac/README.md.
# Apple's SDK and clang's Mac runtime in RC2K_MAC_TOOLS (default ~/.cache/rc2k_mac, filled by
# fetch_tools.sh).
_cross=$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)
export RC2K_MAC_TOOLS=${RC2K_MAC_TOOLS:-$HOME/.cache/rc2k_mac}
case "${1:-x64}" in
    x64) export RC2K_MAC_ARCH=x86_64 ;;
    a64) export RC2K_MAC_ARCH=arm64 ;;
    *) echo "env.sh: x64 or a64" >&2; return 1 ;;
esac
export SDKROOT=${SDKROOT:-$(ls -d "$RC2K_MAC_TOOLS"/MacOSX*.sdk 2>/dev/null | sort -V | tail -1)}
export MACOSX_DEPLOYMENT_TARGET=${MACOSX_DEPLOYMENT_TARGET:-11.0}
_triple=$([ "$RC2K_MAC_ARCH" = arm64 ] && echo aarch64_apple_darwin || echo x86_64_apple_darwin)
_TRIPLE=$(echo "$_triple" | tr a-z A-Z)
export "CARGO_TARGET_${_TRIPLE}_LINKER=$_cross/cc.sh"
export "CC_$_triple=$_cross/cc.sh" "CXX_$_triple=$_cross/cc.sh" "AR_$_triple=llvm-ar"
export "CMAKE_TOOLCHAIN_FILE_$_triple=$_cross/toolchain.cmake"
unset _cross _triple _TRIPLE
