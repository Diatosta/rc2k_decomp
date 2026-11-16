#!/usr/bin/env bash
# tools/mac/cross/fetch_tools.sh --accept-apple-license: Apple's macOS SDK and clang's Mac
# runtime (libclang_rt.osx.a) for the Linux-hosted Mac build, from the Command Line Tools for
# Xcode in Apple's software update catalogue (product 140-17812: SDK 26.5), into RC2K_MAC_TOOLS
# (default ~/.cache/rc2k_mac). Both are Apple's, under the Xcode and Apple SDKs licence: the
# flag says the one running this has accepted it. Needs curl, bsdtar, bsdcpio, python3.
set -euo pipefail
[ "${1:-}" = --accept-apple-license ] || {
    echo "fetch_tools.sh: Apple's SDK is under the Xcode and Apple SDKs Agreement;" >&2
    echo "run with --accept-apple-license once you have accepted it." >&2
    exit 1
}
TOOLS=${RC2K_MAC_TOOLS:-$HOME/.cache/rc2k_mac}
PRODUCT=140-17812
CATALOG=https://swscan.apple.com/content/catalogs/others/index-26-15-14-13-12-10.16-10.15-10.14-10.13-10.12-10.11-10.10-10.9-mountainlion-lion-snowleopard-leopard.merged-1.sucatalog.gz
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT
mkdir -p "$TOOLS/clang_rt"

curl -fsS --max-time 300 -o "$WORK/catalog.gz" "$CATALOG"
# A package's address in the product, by the end of its name.
package() {
    python3 -I - "$WORK/catalog.gz" "$PRODUCT" "$1" <<'EOF'
import gzip, plistlib, sys
catalog = plistlib.loads(gzip.decompress(open(sys.argv[1], 'rb').read()))
print(next(p['URL'] for p in catalog['Products'][sys.argv[2]]['Packages']
           if p['URL'].endswith(sys.argv[3])))
EOF
}
# A package's payload (pbzx: xz chunks) as a cpio archive.
payload() {
    (cd "$WORK" && bsdtar -xf "$1" Payload && mv Payload "$1.payload")
    python3 -I - "$WORK/$1.payload" "$WORK/$1.cpio" <<'EOF'
import lzma, struct, sys
f = open(sys.argv[1], 'rb')
assert f.read(4) == b'pbzx'
f.read(8)
with open(sys.argv[2], 'wb') as out:
    while (head := f.read(16)) and len(head) == 16:
        _, size = struct.unpack('>QQ', head)
        chunk = f.read(size)
        out.write(lzma.decompress(chunk) if chunk[:6] == b'\xfd7zXZ\x00' else chunk)
EOF
}

curl -fsS --max-time 1800 -o "$WORK/sdk.pkg" "$(package CLTools_macOSNMOS_SDK.pkg)"
payload sdk.pkg
(cd "$WORK" && mkdir sdk && cd sdk && bsdcpio -idm --quiet < ../sdk.pkg.cpio)
SDK=$(find "$WORK/sdk" -maxdepth 5 -type d -name 'MacOSX[0-9]*.sdk' | sort -V | tail -1)
rm -rf "$TOOLS/$(basename "$SDK")"
mv "$SDK" "$TOOLS/"

curl -fsS --max-time 3000 -o "$WORK/exe.pkg" "$(package CLTools_Executables_Universal.pkg)"
payload exe.pkg
(cd "$WORK" && mkdir exe && cd exe && bsdcpio -idm --quiet '*lib/clang/*/lib/darwin/*' < ../exe.pkg.cpio)
cp "$(find "$WORK/exe" -name libclang_rt.osx.a | head -1)" "$TOOLS/clang_rt/"
echo "fetch_tools.sh: $TOOLS/$(basename "$SDK"), $TOOLS/clang_rt/libclang_rt.osx.a"
