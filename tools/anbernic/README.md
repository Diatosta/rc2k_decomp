# Anbernic RG34XXSP

Stock OS with the community mod: Ubuntu 22.04 arm64, glibc 2.35, kernel 4.9, no DRM.
`src/handheld.rs` shows the frames on `/dev/fb0`; SDL does the sound and the pad.

- Container: `docker build -t rc2k-anbernic tools/anbernic` (`run.sh` runs a command in it).
- Deploy: `./deploy_standalone.sh anbernic`; `ANBERNIC_HOST=root@address` in `deploy.local`,
  the key in `~/.ssh/rg34xxsp` (or `ANBERNIC_KEY`).
- First time, the game's data: from an installed game's folder,
  `tar cf - files var savedata | ssh root@address 'mkdir -p /mnt/mmc/Roms/APPS/rally; tar xf - -C /mnt/mmc/Roms/APPS/rally'`
  ("Cannot change ownership" on the SD card is harmless).
- On the device: Apps > Rally. MENU + START leaves. Log: `Roms/APPS/rally/rc2k.log`.
- GPU: `RC2K_RENDERER=gpu` (set by `Rally.sh`) draws races through libmali's OpenGL ES
  (`src/render/gles.rs`); without it the rasteriser draws them at half size.
- Profiling: `RC2K_PROFILE=1` (set by `Rally.sh` for now) writes `profile.txt`: the marker's
  place (`handheld::start_profile`), then (place, thread, five callers) a line; resolve against the
  unstripped build with `aarch64-linux-gnu-addr2line -i -f -C`.
- Testing over SSH: stop `loadapp.sh` and `dmenu_ln` (SIGSTOP), kill `dmenu.bin`; SIGCONT brings
  the launcher back. Writing `input_event`s to `/dev/input/event1` presses buttons.
- Automated runs: `tools/remote.py build --handheld`, then `start`, `pad`, `key`, `shot`, `state`, `stop` with `--handheld`.
