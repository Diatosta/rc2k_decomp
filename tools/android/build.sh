#!/usr/bin/env bash
# tools/android/build.sh: the Android .apk, from the tools in $RC2K_ANDROID_HOME (see
# tools/android/README.md). Usage: tools/android/build.sh [x86_64] [arm64] [--debug]
# (no ABI: both). The package is debuggable (adb run-as, for pushing the game's files) unless
# --final. Writes dist/android/rc2k-<abi>.apk, signed with a debug key kept in
# $RC2K_ANDROID_HOME/debug.keystore, or with the release key when RC2K_ANDROID_KEYSTORE names its
# keystore (RC2K_ANDROID_KEY_PASSWORD its password, RC2K_ANDROID_KEY_ALIAS its alias, default
# rc2k: the CI's release builds). Packaged by hand with the SDK's own tools (javac, d8, aapt2,
# zipalign, apksigner): no Gradle.
set -euo pipefail
cd "$(dirname "$0")/../.."
source tools/android/env.sh

abis=(); profile=release; debuggable=(--debug-mode)
for a in "$@"; do
    case "$a" in
        --debug) profile=debug ;;
        --final) debuggable=() ;;
        x86_64|arm64) abis+=("$a") ;;
        *) echo "usage: $0 [x86_64] [arm64] [--debug] [--final]" >&2; exit 1 ;;
    esac
done
[ ${#abis[@]} -gt 0 ] || abis=(x86_64 arm64)

BT=$ANDROID_HOME/build-tools/35.0.1
JAR=$ANDROID_HOME/platforms/android-35/android.jar
SDL_JAVA=$(echo ~/.cargo/registry/src/*/sdl3-src-3.4.18/SDL/android-project/app/src/main/java)
WORK=${RC2K_ANDROID_WORK:-$RC2K_ANDROID_HOME/work}
OUT=dist/android
export CARGO_TARGET_DIR=${CARGO_TARGET_DIR:-$PWD/target/android}
mkdir -p "$WORK" "$OUT"

# One SDL for every ABI (tools/android/sdl.sh) and the activity's Java compiled once.
for abi in "${abis[@]}"; do
    d=$([ "$abi" = arm64 ] && echo arm64-v8a || echo "$abi")
    [ -f "$RC2K_ANDROID_HOME/sdl3build/$d/libSDL3.so" ] || tools/android/sdl.sh "$abi"
done

rm -rf "$WORK/classes" "$WORK/dex" "$WORK/res" && mkdir -p "$WORK/classes" "$WORK/dex"
javac --release 11 -nowarn -classpath "$JAR" -d "$WORK/classes" \
    "$SDL_JAVA"/org/libsdl/app/*.java tools/android/app/java/org/rc2k/rally/*.java
"$BT/d8" --lib "$JAR" --min-api 26 --output "$WORK/dex" $(find "$WORK/classes" -name '*.class')
cp -r tools/android/app/res "$WORK/res"
python3 tools/android/make_icon.py "$WORK/res"
"$BT/aapt2" compile --dir "$WORK/res" -o "$WORK/res.zip"

if [ -n "${RC2K_ANDROID_KEYSTORE:-}" ]; then
    : "${RC2K_ANDROID_KEY_PASSWORD:?the password of the release keystore}"
    signing=(--ks "$RC2K_ANDROID_KEYSTORE" --ks-key-alias "${RC2K_ANDROID_KEY_ALIAS:-rc2k}"
        --ks-pass env:RC2K_ANDROID_KEY_PASSWORD --key-pass env:RC2K_ANDROID_KEY_PASSWORD)
else
    [ -f "$RC2K_ANDROID_HOME/debug.keystore" ] || keytool -genkeypair -keystore "$RC2K_ANDROID_HOME/debug.keystore" \
        -storepass android -keypass android -alias rc2k -keyalg RSA -keysize 2048 -validity 10000 \
        -dname "CN=rc2k debug" >/dev/null
    signing=(--ks "$RC2K_ANDROID_HOME/debug.keystore" --ks-pass pass:android --key-pass pass:android)
fi

for abi in "${abis[@]}"; do
    d=$([ "$abi" = arm64 ] && echo arm64-v8a || echo "$abi")
    android_cargo_env "$abi"
    cargo build -p rc2k_android $([ $profile = release ] && echo --release)
    so=$CARGO_TARGET_DIR/$CARGO_BUILD_TARGET/$profile/libmain.so
    pk=$WORK/pack-$abi && rm -rf "$pk" && mkdir -p "$pk/lib/$d"
    cp "$so" "$pk/lib/$d/libmain.so"
    cp "$RC2K_ANDROID_HOME/sdl3build/$d/libSDL3.so" "$pk/lib/$d/"
    "$BT/aapt2" link --manifest tools/android/app/AndroidManifest.xml -I "$JAR" \
        --min-sdk-version 26 --target-sdk-version 35 ${debuggable[@]+"${debuggable[@]}"} "$WORK/res.zip" \
        -o "$WORK/base-$abi.apk"
    cp "$WORK/dex/classes.dex" "$pk/"
    # The libraries stored, not compressed: the system maps them from the package.
    python3 -I - "$WORK/base-$abi.apk" "$pk" <<'PY'
import os, sys, zipfile
apk, root = sys.argv[1:]
with zipfile.ZipFile(apk, "a") as z:
    for dirpath, _, names in os.walk(root):
        for n in sorted(names):
            full = os.path.join(dirpath, n)
            rel = os.path.relpath(full, root)
            z.write(full, rel, compress_type=zipfile.ZIP_STORED if rel.endswith(".so") else zipfile.ZIP_DEFLATED)
PY
    "$BT/zipalign" -f -P 16 4 "$WORK/base-$abi.apk" "$WORK/aligned-$abi.apk"
    "$BT/apksigner" sign "${signing[@]}" --out "$OUT/rc2k-$abi.apk" "$WORK/aligned-$abi.apk"
    ls -l "$OUT/rc2k-$abi.apk"
done
