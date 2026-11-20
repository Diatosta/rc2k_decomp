#!/bin/bash
# The iOS C compiler and linker for the Linux-hosted build (tools/ios/README.md): clang for iOS
# (arm64, the device) against the SDK in SDKROOT, linked by ld64.lld with clang's iOS runtime
# (`@available` needs its __isPlatformVersionAtLeast) when RC2K_IOS_TOOLS holds it.
set -eu
SDK=${SDKROOT:?SDKROOT: the iPhoneOS SDK (source tools/ios/env.sh)}
TOOLS=${RC2K_IOS_TOOLS:-$HOME/.cache/rc2k_ios}
args=() link=1 skip=
for a in "$@"; do
    # The target below names the architecture.
    [ "$a" = "-arch" ] && skip=1 && continue
    [ -n "$skip" ] && skip= && continue
    [ "$a" = -c ] && link=
    args+=("$a")
done
extra=()
[ -n "$link" ] && [ -f "$TOOLS/clang_rt/libclang_rt.ios.a" ] && extra=("$TOOLS/clang_rt/libclang_rt.ios.a")
exec clang -target "arm64-apple-ios${IPHONEOS_DEPLOYMENT_TARGET:-13.0}" -isysroot "$SDK" \
    -fuse-ld=lld "${args[@]}" "${extra[@]}"
