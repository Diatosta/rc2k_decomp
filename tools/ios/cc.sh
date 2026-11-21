#!/bin/bash
# The iOS C compiler and linker for the Linux-hosted build (tools/ios/README.md): clang for iOS
# (arm64, the device) against the SDK in SDKROOT, linked by ld64.lld with clang's iOS runtime
# (`@available` needs its __isPlatformVersionAtLeast) from RC2K_IOS_TOOLS or xtool's SDK.
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
if [ -n "$link" ]; then
    # The runtime kept beside the SDK, else the one in xtool's Darwin Swift SDK (where env.sh also
    # looks for the SDK).
    for rt in "$TOOLS/clang_rt/libclang_rt.ios.a" \
        "$HOME"/.swiftpm/swift-sdks/darwin.artifactbundle/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/clang/*/lib/darwin/libclang_rt.ios.a; do
        [ -f "$rt" ] && extra=("$rt") && break
    done
fi
exec clang -target "arm64-apple-ios${IPHONEOS_DEPLOYMENT_TARGET:-13.0}" -isysroot "$SDK" \
    -fuse-ld=lld "${args[@]}" "${extra[@]}"
