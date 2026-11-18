# source tools/ios/env.sh: the environment for `cargo build --target aarch64-apple-ios` on
# Linux (tools/ios/README.md). Apple's iOS SDK (iPhoneOS*.sdk, out of Xcode's .xip) is searched
# in RC2K_IOS_TOOLS (default ~/.cache/rc2k_ios), then in xtool's Darwin Swift SDK. SDKROOT
# names it. RC2K_IOS_TOOLS/clang_rt/libclang_rt.ios.a is clang's iOS runtime from the same Xcode.
_cross=$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" && pwd)
export RC2K_IOS_TOOLS=${RC2K_IOS_TOOLS:-$HOME/.cache/rc2k_ios}
if [ -z "${SDKROOT:-}" ]; then
    SDKROOT=$(ls -d "$RC2K_IOS_TOOLS"/iPhoneOS*.sdk "$HOME"/.swiftpm/swift-sdks/darwin.artifactbundle/*/iPhoneOS*.sdk \
        2>/dev/null | sort -V | tail -1)
fi
if [ -z "$SDKROOT" ] || [ ! -d "$SDKROOT" ]; then
    echo "env.sh: no iPhoneOS SDK in $RC2K_IOS_TOOLS or xtool's (tools/ios/README.md, step 2)" >&2
    return 1
fi
export SDKROOT
export IPHONEOS_DEPLOYMENT_TARGET=${IPHONEOS_DEPLOYMENT_TARGET:-13.0}
export CARGO_TARGET_AARCH64_APPLE_IOS_LINKER="$_cross/cc.sh"
export CC_aarch64_apple_ios="$_cross/cc.sh" CXX_aarch64_apple_ios="$_cross/cc.sh" AR_aarch64_apple_ios=llvm-ar
export CMAKE_TOOLCHAIN_FILE_aarch64_apple_ios="$_cross/toolchain.cmake"
unset _cross
