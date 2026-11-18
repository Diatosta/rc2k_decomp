# The iOS build (iPhone and iPad)

Built, signed and installed from Linux, with no Mac, simulator or virtual machine: the owner's
iPhone is the test. State on 2026-11-18: the program passes `cargo check --target
aarch64-apple-ios` (and `-sim`) without the SDK; the link, the app and the phone are not yet
run (they need Apple's SDK, step 2).

What the program does on iOS (`target_os = "ios"`): SDL3's UIKit and Metal backends (SDL is
built from its source and linked in, `--features ios-sdl-source`); the GPU renderer on Metal
through SDL GPU with the Metal shaders the Mac build uses; no debug UI, Steam, installer
shortcuts or fault-file extras; the game's folder is the app's Documents folder (shown in the
Files app and to file sharing, `UIFileSharingEnabled`): copy the game's folder, or a zip of it,
there; with nothing there the app says so in a message box and ends. Landscape only, the screen
kept awake. Touch controls: another lane's (leave `platform::input` for it); a game controller
(MFi / Xbox / PlayStation pad) works through SDL's controller support.

## Screen size, icon and name (2026-11-18)

The window is made high-density: its drawing size is the screen's pixels (2556 x 1179 on an
iPhone 15, not the 852 x 393 points). The graphics page's resolution list holds the part of the
whole screen in pixels, landscape, with its
3/4, 2/3 and 1/2; the race starts at the largest of these no wider than 1920 (the menus' canvas
is drawn by the processor and uploaded whole each frame, so the full 2556 would cost frames; pick
the largest in the list for the sharpest image). The race frame covers the whole screen, edge to edge; the HUD's pieces and the mirror keep to the
window's safe area (`SDL_GetWindowSafeArea`: `platform::safe_area`, laid out as on a screen of that
size), so the notch and the home indicator hide nothing of them. A menu's 4:3 page is as tall as the
screen (the home indicator lies over its empty bottom strip) and centred across the safe area, the
blurred background filling the sides.
A start logs "platform: the window is ... points, ... pixels, its safe area ..." and the renderer
line (`SDL's gpu` and "3D on a GPU backend" on Metal) in the log in the Documents folder.
The home-screen name is "Rally Championship" (iOS cuts a long label with an ellipsis; the owner accepts that) and the
name is also in `tools/ios/en.lproj/InfoPlist.strings`, which iOS prefers for the label, so it keeps its space (the owner saw "RallyChampionship" once; xtool's installer, read on GitHub, rewrites only the bundle identifier). The
icons are `assets/icon.png` scaled into `tools/ios/icons/`.

**Re-install (owner):** `./build.sh ios`, then `tools/ios/install.sh`. The saved game stays (the
Documents folder is kept when an app is installed over itself); an options file kept from the
first install holds the old 800 x 600, so the new default applies only after removing
`var/rc2k.toml` (or choose the size on the graphics page). If the icon or name stays old, delete
the app from the home screen and install again (the home screen caches them).

## Why no simulator

Apple's iOS Simulator is a macOS program (it runs the iOS frameworks built for the host's
processor on top of macOS's own kernel and Core Animation); no Linux program, Darling included,
provides it. Darling runs command-line Mac programs with partial frameworks (the Mac build's
README lists what is stubbed); it has no UIKit, no Metal and no SimulatorKit, and Xcode's
command-line tools only help to compile, which clang and ld64.lld do here. So the device is the
real test. (A Mac, real or a VM, could host the Simulator: Big Sur takes Xcode 13.2.1 / iOS 15.2
at most. Not used.)

## The steps, in order (owner)

Packages to install (distribution's own): `clang lld llvm cmake python3 usbmuxd` and
`libimobiledevice` tools (`ideviceinfo`, `idevicepair`; package names `libimobiledevice-utils`
on Debian / Ubuntu, `libimobiledevice` on Arch), `udev` rules for the phone come with usbmuxd.
Rust's targets: `rustup target add aarch64-apple-ios aarch64-apple-ios-sim` (done in this
repository's toolchain). Swift 6.3 or newer from swift.org (xtool needs it).

1. **xtool** (https://github.com/xtool-org/xtool): download its latest AppImage from the
   GitHub releases, rename it `xtool`, `chmod +x`, put it in the PATH. It builds Swift
   packages, signs with an Apple ID and installs over USB (`xtool setup`, `auth`, `sdk`,
   `dev`, `devices`, `install`, `uninstall`, `launch`). Here it is used for the SDK, the
   Apple ID and the install; the app itself is built by `build.sh`.
2. **The SDK and the Apple ID login** (the owner only: not done by anyone else). Sign in
   with the Apple ID at https://developer.apple.com/download/all/ (a free Apple ID is
   enough) and download `Xcode_<version>.xip` (Xcode 16 / 26; about 3 GB; the Apple
   Developer Agreement and the Xcode license apply: Apple's terms limit the SDK to
   Apple-branded hardware, which this Linux build does not honour: the owner decides). Then
   `xtool setup` asks for the path of the `.xip` and extracts the SDK from it on Linux (no Mac
   tools needed) into `~/.swiftpm/swift-sdks/darwin.artifactbundle/`; `swift sdk list` shows
   it. `xtool auth login` asks for the Apple ID and password (and the two-factor code on a
   trusted device) and keeps them in the system's keyring; it makes the development
   certificate. Nothing here is typed into any other program.
   By hand, instead of xtool's extraction: `xar -xf Xcode.xip` / `pbzx` and take
   `Xcode.app/Contents/Developer/Platforms/iPhoneOS.platform/Developer/SDKs/iPhoneOS.sdk` into
   `~/.cache/rc2k_ios/` (`RC2K_IOS_TOOLS`); clang's iOS runtime
   `Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/clang/*/lib/darwin/libclang_rt.ios.a`
   into `~/.cache/rc2k_ios/clang_rt/` (`tools/ios/cc.sh` adds it to the link; `@available`
   needs it). `tools/ios/env.sh` finds the SDK in either place.
3. **Build**: `./build.sh ios` (needs the SDK; clang, lld, cmake) makes
   `dist/ios/Rally Championship.app` and `dist/ios/Rally Championship.ipa`, unsigned.
4. **Developer Mode on the phone (iOS 16 and later)**: the first time a signed development
   app is sent, Settings, Privacy & Security, Developer Mode, switch on, the phone restarts,
   confirm "Turn On" and the passcode. (The switch shows only after a development app has
   been sent to the phone once, or Xcode connected; if absent, run step 5 first, then come
   back.)
5. **Plug the iPhone in by USB**, unlock it, tap "Trust This Computer" and enter the passcode.
   `systemctl status usbmuxd` should run (it starts when the phone is plugged in);
   `idevicepair pair` and `ideviceinfo` answer. Wi-Fi pairing exists on Apple's side (the phone
   shown "on Wi-Fi" once paired with a computer by cable) and usbmuxd can reach such a phone
   on the same network, but it was not tried here: use the USB cable.
6. **Sign and install**: `tools/ios/install.sh` (`xtool devices`, `xtool install`). xtool
   registers an App ID (the bundle identifier in `Info.plist`, which a free account may
   have to change to a unique one) and a development profile for the Apple ID, signs the app
   and sends it. Whether `xtool install` takes a ready-made `.ipa` (not a Swift package) was
   not tried: if it refuses, the alternative is `zsign` with the certificate and profile
   (`zsign -k key.pem -m profile.mobileprovision -o signed.ipa unsigned.ipa`) and
   `ideviceinstaller -i signed.ipa`, or sideloading the `.ipa` with SideStore / AltStore.
7. **Trust the developer profile on the phone**: Settings, General, VPN & Device Management,
   the Apple ID under "Developer App", Trust. Then start the app.
8. **The game's data**: Files app, On My iPhone, Rally Championship (the app's Documents):
   copy the game's folder (or a zip of it) in; or from Linux
   `idevicepair`/`ifuse --documents io.github.diatosta.rc2k ~/mnt` (package `ifuse`) and
   copy there. Start the app again.

## The free signing's limits

A free Apple ID's development profile lasts 7 days; after that the app no longer opens
(its data in Documents stays). Install again with step 6 (the same command; weekly, or a
sideloader's automatic refresh such as SideStore). At most 3 apps with a free account's
profiles on a device at a time, and 10 new App IDs a week per Apple ID. A paid Developer
Program account ($99 a year) lasts a year.

## Not tried yet

The link, the Metal shaders on an iOS GPU (they are the Mac build's MSL 2.1; spirv-cross has
an `--msl-ios` platform switch if a shader misbehaves), the app's start, backgrounding (the
program treats `WILL_ENTER_BACKGROUND` as losing focus), memory limits (a phone kills an app
using far more than its share: the zip is unpacked entry by entry in memory, and a 1 GB entry
is refused), the Files app's folder layout, controllers and the screen's safe area.
