#!/bin/bash
# The Mac C compiler and linker for the Linux-hosted Mac build (tools/mac/README.md): clang for
# macOS on RC2K_MAC_ARCH against the SDK in SDKROOT, linked by ld64.lld with clang's Mac
# runtime from Apple's Command Line Tools (`@available` needs its __isPlatformVersionAtLeast;
# Linux's clang has none for Darwin).
# RC2K_MAC_WEAK_FRAMEWORKS=1, test builds for Darling only: every framework weak (Darling's
# lack some of what SDL imports: the program starts and a missing piece is null) and Darling's
# broken calls stood in for (darling_shim.c).
SDK=${SDKROOT:?SDKROOT: a macOS SDK (tools/mac/cross/env.sh)}
ARCH=${RC2K_MAC_ARCH:-x86_64}
TOOLS=${RC2K_MAC_TOOLS:-$HOME/.cache/rc2k_mac}
args=() link=1
for a in "$@"; do
    # The target below names the architecture.
    [ "$a" = "-arch" ] && skip=1 && continue
    [ -n "$skip" ] && skip= && continue
    [ "$a" = -c ] && link=
    [ -n "$RC2K_MAC_WEAK_FRAMEWORKS" ] && [ "$a" = -framework ] && a=-weak_framework
    args+=("$a")
done
extra=()
if [ -n "$link" ]; then
    extra=("$TOOLS/clang_rt/libclang_rt.osx.a")
    if [ -n "$RC2K_MAC_WEAK_FRAMEWORKS" ]; then
        # The frameworks the others re-export (Cocoa: AppKit; AppKit and Carbon:
        # ApplicationServices: CoreGraphics, CoreServices) load strong unless named weak first.
        args=(-weak_framework ApplicationServices -weak_framework AppKit -weak_framework CoreGraphics
              -weak_framework CoreServices -weak_framework CoreFoundation "${args[@]}")
        extra+=("$(dirname "$0")/darling_shim.c")
    fi
fi
exec clang -target "$ARCH-apple-macos${MACOSX_DEPLOYMENT_TARGET:-11.0}" -isysroot "$SDK" \
    -fuse-ld=lld "${args[@]}" "${extra[@]}"
