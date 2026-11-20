# Android

*Category: [Getting started](Getting-started)*

A build for Android phones and tablets (64-bit ARM, and x86-64 for the emulator). It runs on a
phone: the menus and races by touch or a pad, vibration, screenshots into an album, replays,
being sent to the background and back (on a 120 Hz phone the menus run at 120 frames a second).
Not yet run: Android 9 and older. The [iOS](iOS) build shares its touch controls.

## What it is

An app (`org.rc2k.rally`) made of SDL's activity and the program as a shared library
(`libmain.so`). The game, the rasteriser and the GPU renderer are the same as on the other
systems; the GPU one runs on Vulkan, with the software rasteriser as the fallback. The window
covers the whole screen (no system bars) in landscape, and the game's picture keeps its shape in
it. The package has no pictures of the game: its icon is our own placeholder.

## The game's files

The app does not install the game. The game's folder (`files`, and `var` and `savedata` if you
have them) goes into the app's own storage, `files/game`. For testing, from a computer with
`adb`, with a build made debuggable (`tools/android/build.sh --debuggable`; the default build is not, so that no other app with `adb` rights can read its storage):

```
tools/android/push_game.sh <the game's folder>
```

Words the program should start with (`--stage langton --gearbox automatic`, for a test) are read
from `files/rc2k_args.txt` in the app's storage.

The game folder on a phone is the installation: its sound plays whether or not the folder has the
installer's record (`var\ral.zog`; without it the 1999 game took itself to be running from the CD
and switched the sound off). Films are shown with black bars beside them, not the menus' blurred
background.

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
| + and - | gear up and down (a second pair on the left, under II, for the other hand) |
| CAM | the camera |
| LIGHTS (under CAM) | the lights on and off |
| II | the pause: asks whether to leave the stage |

The controls are a virtual gamepad ("Touch screen") the game reads like any other, so its
profile on the controller page works as a pad's does. Fingers work together, and a finger sliding
from one pedal to the next presses the next. The layout keeps out of the screen's notch and
rounded corners (SDL's safe area).

In the menus: a tap chooses the item under the finger; a drag up or down moves along the list; a
drag sideways steps a value (and turns the main menu's carousel); BACK in the corner, two fingers
tapped together, or Android's back button go back. The text fields use the system's keyboard.

## Building

Everything the build needs lives in one folder (named by `RC2K_ANDROID_HOME`, for example
`~/android`): the SDK's command-line tools, platform tools, the NDK, a platform and its
build tools, the emulator and a system image (installed with `sdkmanager`), and the Rust targets
`aarch64-linux-android` and `x86_64-linux-android`. Besides: a JDK, CMake, and Python 3 with
Pillow (the app's icons are drawn by `tools/android/make_icon.py`). `tools/android/` has the scripts:

```
tools/android/build.sh x86_64 arm64      # dist/android/rc2k-x86_64.apk, rc2k-arm64.apk
```

It builds SDL3 (the same source the Rust build uses) with the NDK's CMake toolchain, the program
with cargo for each ABI, SDL's Java glue and our activity with `javac` and `d8`, and packages and
signs the `.apk` with the SDK's `aapt2`, `zipalign` and `apksigner` (a debug key; no Gradle). A
build is not debuggable unless asked for with `--debuggable` (for `push_game.sh`).

### The release key

An app can only be installed over an earlier one signed with the same key; one signed with
another key has to be uninstalled first, and uninstalling deletes the game's files and saves in
the app's storage. So the released packages are all signed with one release key, kept as three
secrets of the GitHub repository; the `android` job of the automatic builds signs with it on a
push (and with a debug key, with a warning, on a pull request or when the secrets are missing: a
pull request never sees the secrets). Locally, `RC2K_ANDROID_KEYSTORE`
(the keystore file), `RC2K_ANDROID_KEY_PASSWORD` and `RC2K_ANDROID_KEY_ALIAS` (default `rc2k`)
make `tools/android/build.sh` sign with it too.

Made once, with the JDK's `keytool` (it asks for a password and for the name fields, which may be
anything):

```
keytool -genkeypair -keystore rc2k-release.jks -storetype PKCS12 -alias rc2k \
    -keyalg RSA -keysize 4096 -validity 10000
```

Then in the repository, Settings, Secrets and variables, Actions, New repository secret (or with
`gh secret set NAME`):

| Secret | Value |
| --- | --- |
| `ANDROID_KEYSTORE_BASE64` | the keystore in base64: `base64 -w0 rc2k-release.jks` |
| `ANDROID_KEYSTORE_PASSWORD` | the password given to `keytool` |
| `ANDROID_KEY_ALIAS` | `rc2k` |

Keep the keystore file and its password somewhere safe besides GitHub (a password manager, a
backup): GitHub never shows a secret again, and a lost key means every player has to uninstall
the app (losing its saves) to install the next one.

## The speed, and the log

The log (`rc2k.log`, in the app's `files/game`) has a `phone:` line every five seconds during a
race: frames a second, the screen's size, the 3D's share, the renderer, the display's refresh rate
and where a frame's time goes ([Log files](Log-files)). The advanced page's last row, "share
log", sends the log through Android's share sheet. The renderer row offers the GPU (Vulkan) and
the software rasteriser; the OpenGL backend of the desktop needs a window of its own and a phone
has one, so it is not offered. A GL ES backend on the game's own window is not built yet.

### When the phone's GPU is refused

The GPU device is made at the start, before the race's size is chosen. SDL's Vulkan driver is told
to do without three features the game never uses (shader clip distance, anisotropic filtering,
indirect draws with a first instance), which some phones' drivers (Mali) do not offer, so they no
longer keep a phone out. Depth clamping the scene's pipeline does use: a phone whose driver lacks
it gets a device without it and the pipelines clip instead. If SDL still finds no device, the log
says why (SDL's own warnings and errors for the GPU, video and render categories are written to
`rc2k.log` on Android, as `sdl[category N, priority M]: ...` lines) and the 3D goes on the software
rasteriser. The race is then at most 1280 pixels wide (the screen's ratio kept) unless a size was
set by hand in `rc2k.toml`, since the software rasteriser cannot carry a phone's full screen; the
advanced page's renderer row then reads "software (no GPU)". The rasteriser already uses every
core of the phone (rayon's pool); the "drawing threads" count in the `phone:` line is the scene's
parts, at most four (a view, its mirror, split screen's views), which a single player's race
does not fill.

## Not done yet

- a run on Android 9 and older (the screenshots' album takes another way there);
- a first-run picker for the game's zip, and a play-store style signed package;
- the options pages that make no sense on a phone (window mode, the keyboard's keys);
- the DualSense/Xbox pad's extras, Steam, and the debug window are left out of this build.
