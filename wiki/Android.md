# Android

*Category: [Getting started](Getting-started)*

A build for Android phones and tablets (64-bit ARM, and x86-64 for the emulator). It is **begun**:
it builds, starts, shows the menus, plays a race with touch controls and survives being sent to
the background and back. It has been run in the Android emulator only, not yet on a phone.

## What it is

An app (`org.rc2k.rally`) made of SDL's activity and the program as a shared library
(`libmain.so`). The game, the rasteriser and the GPU renderer are the same as on the other
systems; the GPU one runs on Vulkan, with the software rasteriser as the fallback. The window
covers the whole screen (no system bars) in landscape, and the game's picture keeps its shape in
it. The package has no pictures of the game: its icon is our own placeholder.

## The game's files

The app does not install the game. The game's folder (`files`, and `var` and `savedata` if you
have them) goes into the app's own storage, `files/game`. For testing, from a computer with
`adb`, with a build that is debuggable (the default of `tools/android/build.sh`):

```
tools/android/push_game.sh <the game's folder>
```

Words the program should start with (`--stage langton --gearbox automatic`, for a test) are read
from `files/rc2k_args.txt` in the app's storage.

A first-run page with a file picker for the game's zip is to come; the page the other systems show
(pick a folder or a zip) is there but its file dialog is not tried on Android.

## Touch controls

In the race (the overlay over the picture; a real gamepad being used hides it, a finger brings it back):

| Control | Does |
| --- | --- |
| a finger down in the left part, moved left or right | steers, as an analogue wheel; past the lock the middle follows the finger |
| GO | accelerates |
| BRAKE | brakes |
| HAND | handbrake |
| + and - | gear up and down |
| CAM | the camera |
| II | the pause: asks whether to leave the stage |

The controls are a virtual gamepad ("Touch screen") the game reads like any other, so its
profile on the controller page works as a pad's does. Fingers work together, and a finger sliding
from one pedal to the next presses the next. The layout keeps out of the screen's notch and
rounded corners (SDL's safe area).

In the menus: a tap chooses the item under the finger; a drag up or down moves along the list; a
drag sideways steps a value (and turns the main menu's carousel); BACK in the corner, two fingers
tapped together, or Android's back button go back. The text fields use the system's keyboard.

## Building

Everything the build needs lives in one folder (`/mnt/discoIII/android` here; set
`RC2K_ANDROID_HOME`): the SDK's command-line tools, platform tools, the NDK, a platform and its
build tools, the emulator and a system image (installed with `sdkmanager`), and the Rust targets
`aarch64-linux-android` and `x86_64-linux-android`. `tools/android/` has the scripts:

```
tools/android/build.sh x86_64 arm64      # dist/android/rc2k-x86_64.apk, rc2k-arm64.apk
```

It builds SDL3 (the same source the Rust build uses) with the NDK's CMake toolchain, the program
with cargo for each ABI, SDL's Java glue and our activity with `javac` and `d8`, and packages and
signs the `.apk` with the SDK's `aapt2`, `zipalign` and `apksigner` (a debug key; no Gradle). A
build with `--final` is not debuggable.

## Not done yet

- a run on a real phone, and the speed there (the emulator draws on a software Vulkan: about 9
  frames a second);
- a first-run picker for the game's zip, and a play-store style signed package;
- the options pages that make no sense on a phone (window mode, the keyboard's keys);
- the DualSense/Xbox pad's extras, Steam, and the debug window are left out of this build.
