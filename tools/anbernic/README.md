# Anbernic RG34XXSP

Knulli (Batocera-based; since 2026-11-15): Anbernic's kernel 4.9 with libmali, no DRM, glibc 2.40.
The build is made for the stock OS's Ubuntu 22.04 (glibc 2.35) and needs only the C library, so it
runs on either. `src/handheld.rs` shows the frames on `/dev/fb0`; SDL does the sound and the pad.

- Container: `docker build -t rc2k-anbernic tools/anbernic` (`run.sh` runs a command in it).
- Deploy: `./deploy_standalone.sh anbernic`; `ANBERNIC_HOST=root@address` in `deploy.local`,
  the key in `~/.ssh/rg34xxsp` (or `ANBERNIC_KEY`). The game's folder is `/userdata/roms/ports/rally`
  (`ANBERNIC_GAME` for another; the stock OS's was `/mnt/mmc/Roms/APPS/rally`), its entry
  `roms/ports/Rally.sh` beside it.
- First time, the game's data: from an installed game's folder,
  `tar cf - files var savedata | ssh root@address 'mkdir -p /userdata/roms/ports/rally; tar xf - -C /userdata/roms/ports/rally'`
  ("Cannot change ownership" on the SD card is harmless).
- On the device: Ports > Rally. MENU + START leaves. Log: `roms/ports/rally/rc2k.log`.
- GPU: races are drawn through libmali's OpenGL ES (`src/render/gles.rs`) by default; if it
  cannot start, the log says why and the rasteriser draws them at half size. `RC2K_RENDERER=software`
  (`remote.py start --software`) forces the rasteriser. `Rally.sh` sets nothing.
- Profiling: `RC2K_PROFILE=1` (`remote.py start --profile`; not set by `Rally.sh`: it costs
  frames) writes `profile.txt` every 5 s, on a thread of its own: the marker's
  place (`handheld::start_profile`), then (place, thread, 24 callers) a line; resolve against the
  unstripped build with `aarch64-linux-gnu-addr2line -i -f -C`.
- Testing over SSH: `remote.py` pauses the front end (SIGSTOP: Knulli's `emulationstation`, or the
  stock OS's `loadapp.sh` and `dmenu_ln`, `dmenu.bin` killed); SIGCONT brings it back. Writing `input_event`s to `/dev/input/event1` presses buttons.
- Automated runs: `tools/remote.py build --handheld`, then `start`, `pad`, `key`, `shot`, `state`, `stop` with `--handheld`.
- Profile-guided build (2 fps more, about 6 %): `tools/anbernic/pgo.sh` builds the game instrumented,
  drives it on the handheld (`pgo_train.py`: menus, then the first stage by the pilot) and writes
  `target/a64-pgo/rc2k.profdata`; `./deploy_standalone.sh anbernic pgo` builds with it. Train
  again after larger changes (changed functions are built as without a profile). Needs
  `llvm-profdata` of rustc's LLVM major version.
