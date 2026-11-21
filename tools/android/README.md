# Android build

See the wiki page `wiki/Android.md` for what the port does. Here: the tools.

- `env.sh` the places of everything (`RC2K_ANDROID_HOME`, default `~/.local/share/rc2k_android`) and
  `android_cargo_env <x86_64|arm64>` (cargo's linker and compilers from the NDK).
- `sdl.sh <abi>` SDL3's shared library for an ABI (build.sh calls it when missing).
- `build.sh [x86_64] [arm64] [--debug] [--debuggable]` the `.apk` into `dist/android/` (not debuggable unless `--debuggable`).
- `push_game.sh <game folder>` the game's files into the app's private storage (needs a build made with `--debuggable`).
- `make_icon.py` writes the placeholder launcher icons.
- `app/` the manifest, resources and the activity class.

Installed once with `sdkmanager` (licences accepted): `platform-tools`, `platforms;android-35`,
`build-tools;35.0.1`, `ndk;28.2.13676358`, `emulator`, `system-images;android-35;google_apis;x86_64`;
and `rustup target add aarch64-linux-android x86_64-linux-android`.

The emulator, for tests without a window or sound (no window on the desktop):

```
ANDROID_AVD_HOME=$RC2K_ANDROID_HOME/avd ANDROID_USER_HOME=$RC2K_ANDROID_HOME/user \
  emulator -avd rc2k35 -no-window -no-audio -no-boot-anim -no-snapshot -gpu swiftshader_indirect
adb install -r dist/android/rc2k-x86_64.apk
adb exec-out screencap -p > shot.png
adb logcat -s rc2k          # the program's first words; its own log is files/game/rc2k.log
```
