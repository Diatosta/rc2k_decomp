# Code review: readability and maintainability (2026-11-10)

Owner, 2026-11-10: after F7, look at the whole code for what can be improved and modernised,
as F7 did for the drawing. Five read-only Sonnet reviewers surveyed the codebase by area;
their findings, with locations, proposals, effort and risk, are in `notes/review/`:

| Area | File | Batches |
| --- | --- | --- |
| race, drive, HUD (+ core) | `review/race_drive_hud.md` | 11 |
| effects, scene, render, d3d (+ core) | `review/effects_scene_render.md` | 13 |
| menus, screens, saves, records, settings, text, strings (+ core) | `review/menus_screens.md` | A .. J |
| physics, math, sound, audio, input (+ core) | `review/physics_math_sound_input.md` | B1 .. B14 |
| infrastructure: memory model, pointers, files, net, install, debug, os (+ core) | `review/infrastructure.md` | B1 .. B11 |

Caveats:
- The reviewers read the central files and sampled the rest by grep counts. Each claim
  is to be checked before it is acted on (an implementing agent re-reads its files).
- The counts are greps, not builds.
- Checked so far: the determinism finding (W1 below).
- Still to check:
  - `run_ribbons` reading the shared view matrix (the effects draw in the overlays on the
    game thread after `view_drawn` hands the view's matrices on, so likely correct; make
    it an explicit input anyway);
  - `point_into_homes` cited but missing.

Rules for implementing:
- Behaviour-preserving: the held races, replays 00 / 01 and the split runs stay
  bit-identical.
- One check run and one commit per batch; agents in worktrees, disjoint file sets.
- The files F7 is finishing wait for it: `workers.rs`, `race/view_scratch.rs`,
  `race/draw_race_scene.rs`, `race/scene_hand_off.rs`, `race/frame_views.rs`,
  `race/frame_draw/view.rs`, `scene/view_window.rs`, `d3d/render_cache.rs`, `debug/ui.rs`.

## Workstreams across the areas

The same kinds of debt show up in every area; implement them as one workstream each, the
areas as its batches.

**W1. Determinism of the non-IEEE float operations (decision for the owner).** Checked:
- `rcpss` / `rsqrtss` use the CPU's instruction on x86, which `src/arch.rs:32` notes
  differs between processors, and are exact elsewhere.
- `sin` / `cos` / `atan2` come from each platform's maths library (no `libm` crate).

So a replay or a network game may differ between Windows/x86, the handheld (ARM) and the
Mac, or even between two x86 CPUs; the original had the per-CPU part too. Fix: a software
table of Intel's estimates (checked exhaustively once against the hardware), and the
`libm` crate. Small (physics review B1); it changes replays across machines, not on one
machine.

Done (2026-11-10):
- `rc2k_core::math::estimates`: the 5800X's `rcpss` / `rsqrtss` as three 4096-entry
  tables (`estimate_tables.rs`, made by `tools/estimate_tables generate`); `arch::rcpss` /
  `rsqrtss` use them on every machine. `estimate_tables verify`: 0 differences over 2^32
  inputs on the 5800X; on the work machine's i7-8550U the processor differs from them on
  2,183,086,080 (`rcpss`) and 1,133,238,272 (`rsqrtss`) inputs. The two asm tests that run
  the instructions skip where the processor is not the tables'
  (`arch::estimates_are_the_processors`).
- `rc2k_core::math::portable` (libm 0.2, no default features: no processor paths) for
  sin, cos, sin_cos, atan2, log2, exp2, powf, sinf, cosf; `clippy.toml` disallows the std
  methods. A hash of 16M results: libm the same on x86_64 Linux, i686 Windows (Wine) and
  the handheld (aarch64); the platform's the same on the two glibc machines, different
  on Windows. libm is not slower (sin 7.0 against glibc's 8.2 ns, atan2 10.1 against
  11.3).
- Found on the way: `g_useSse` (the estimates' paths) is only set by a developer key,
  so in play the estimates are not reached; but MMX (`has_mmx`) is read directly by the
  lighting (`vertex_light`, `add_point_lights`, `set_object_light`, `car_lighting`,
  `course_faces`), and the handheld reported none: its lighting took the other path.
  `system::test_processor` now reports the same features everywhere (FPU, TSC, CMOV, MMX,
  SSE, no 3DNow!).
- Held races (Langton) on the PC (Ryzen 5800X) and the work machine (i7-8550U) with the
  same `var` / `savedata`: every tick, flush and sound turn identical, before W1 and after
  (the estimates are not reached there, and both use glibc). The screen mode must match:
  the car blocks hold the course map picture's places, which depend on it (a machine
  without the saved mode falls back to 640 x 480).
- The handheld with the MMX paths: 30.7 - 33.6 fps driven on Langton, as before (30.5 - 34).
  Committed 32f829cb; `tools/remote.py` now honours `RC2K_REMOTE_INSTALLED` (3557c6f4).
- Not run: the Mac; a handheld held race against the PC (the handheld's races are
  640 x 480, so the car blocks' screen fields differ: a sim hash without them would be
  needed).

**W2. The original's code and data out of the tests (end of project; owner, 2026-11-10).**
- About 30k lines of test-only original assembly (`asm_reference`, `asm_differential`,
  `x87_reference_*`, 58 + 40 + 19 modules by area).
- About 250 `image_bytes` / `first_values_are_the_games` checks against
  `data/ral_image.bin`.
- `single_rally`'s op-table interpreter.

Freeze each comparison into golden values (seeded CRCs or tables, generated once), then
delete the assembly, the image and the oracles. Reviews: race B1, effects B2, menus E,
physics B13, infrastructure B9. Mechanical per module; L overall.

**W3. Names.**
- IDA-era names: `dword_..` / `flt_..` (about 1,500 in text), `unknown_N` fields (about
  130 + 621 "not named" fields crate-wide), `_gap_` fields (300).
- The marks settings' `listener_switch_N` are really the snow, rain and windscreen counts.
- Files named after addresses (`review_options_48108e_loop`, `*_0`).
- Colliding type names (`Place` ×5, `Window` ×4, `Flags` ×4), and "marks" where the
  module is `effects`.

Progress: round 1 (b2d32058: effects settings, menus' address names, physics' IDA names);
the meanings round (15c5a15a, 2026-11-10: group 3 of the unknowns, the used values traced
writer to reader by five area agents; evidence, leads and slips in `notes/meanings.md`).
Then (61139db7) the tyre gauges named (`tyre_wears`), `computer_ease_speed`, the speed-ups
the meanings found and the wear columns' fix; the name collisions (`BytePlace`,
`FixedBlock`, `RegionCache`, `RecordTail`, `MirrorRect`, `RaceShared`, the five `Flags`) and
a second readings pass (network flag, arcade run kind, `scene_set_up`). Left: the race
regrouping, `turn_point_xz_*`, the sound files, `turn_wheel_angles` (really the mud
accumulation), ~230 "the name is a reading" headers (names from the game's titles).
W7 under way (2026-11-10, four agents): menu item functions as data (A3 / A4), the drivers
and carousel tables compacted, the HUD layouts, `settings_copy` as loops.

Rename maps, mechanical. Reviews: effects B1, menus B, physics B10, infrastructure B3,
race B6.

**W4. Explicit inputs.**
- Global accessors inside logic: `race()` about 223 in race code, `menus()` 98,
  `front_end()` 110, `system()` 68, `marks()`.
- 159 functions in physics and sound take the whole `&Race` for two fields.

Pass the fields. Reviews: race B5, effects B6, menus G, physics B4. With the 1,025
"owner fetches" left for this phase.

**W5. Raw bits to types.**
- About 2,500 `set_bits` / `bits()` uses in effects alone, plus 798 / 493 in race.
- 127 `get() != 0` flags.
- 1.1k `Float::from_bits(0x..)` constants.
- Render-state numbers as an enum.

Make them enums, bools, `f32` fields and named constants. Bit-exact by construction.
Reviews: race B3, effects B7 / B8, physics B6, infrastructure B8.

**W6. x87 and hook-era shapes.**
- The `math::x87*` helpers (about 280 call-site-shaped functions) and the `_x87` twins
  written as `t0 .. tN` temporaries.
- The damage model (5.6k lines of x87 transcription).
- Register-shaped signatures and the sound command queue.
- `_plain` / `_in_game` twins, and `Result<_, Refused>` that cannot fail.

The physics review's x87 section gives the rule:
- In this game's code, rounding is to nearest at double precision, so `X` is `f64` and a
  store `as f32`.
- That is not bit-identical where the code uses chop rounding, the extended precision
  after `finit` (`effects/lines.rs`), `fldpi`, or the estimates.

Proofs: exhaustive, seeded CRC, replay hash. Reviews: physics B2 / B3 / B5 / B7, effects
B3, race B8, infrastructure B7.

**W7. Tables as data, duplication, structure.**
- The menu tables exist twice: typed in core (18k lines) and again in the image's layout
  in `src`, with item actions looked up by RAL address on every click.
- About 60 one-function wrapper files.
- `drivers_store.rs` has 7.4k lines, one number per line.
- `carousel_set_ups.rs` (1.2k lines) should be a table.
- HUD layouts 0 / 1 / 2 are near-identical.
- `settings_copy`'s small virtual machine should be two loops over `GAUGES`.
- Debug rows keyed by IDA address (about 6k generated lines) should be typed accessors.
- `standalone::start` is 219 lines.
- Duplicates: the windscreen drops / flakes and `emit_particle_N`.

Reviews: menus A / I, effects B9 / B11, race B2 / B4 / B6, infrastructure B4 / B6 / B10.

**W8. Screens as states.** The loop is already a state machine (`Next`). Recommended path
(menus review):
1. `Screen::DEFAULT` and sub-structs for the 40-field descriptor.
2. A returned exit value in place of the global result flags.
3. The loop into `rc2k_core` behind a host trait.
4. Optionally, a `ScreenState` trait last.

**W9. Errors and comments.**
- Refusals that can no longer happen.
- Stale hook / shadow-mode / phase text: `rc2k_core/src/lib.rs`, `errors.rs`, comments
  about hooks.
- Comments dense with IDA addresses where a sentence would do.
- A fatal report's register dump that prints zeros.

Mechanical, any time, per module. Reviews: every area's last batches.

First pass done (2026-11-10, five Sonnet agents by area; 32bc34bc, 412e049b): the "The
Rust version" openers and separators, register / flag / liveness notes, phase history,
stale hook and `point_into_homes` mentions, addresses after other functions' names;
`#![allow(clippy::missing_safety_doc)]` removed; two `unwrap`s in records gone. No refusal
removed (every one checked reads a run-time value). Left for a hand pass: register names
inside longer descriptions (~330 in race alone), the IDA data names in headers (714 in the
infrastructure), comments for the tricky logic each review lists, some terse leftovers
("On the variables' cells `v`."), the log strings that still say "the Rust version".

**W10. The memory model (largest; last).** The handheld's further speed waits for W5 and
W10 (owner, 2026-11-11; `notes/conversion.md`, end). The state is still shaped like the 1999 image:
10.3k `Dword`s, 426 `Loose`, `#[repr(C)]` stores at IDA offsets, 2,333 `offset_of!`
checks, the 394-entry `MOVED` table, `GamePtr` / game addresses and the address map.

The infrastructure review's B11 gives the layers, the plain-Rust target (owned structs
passed by reference) and staged steps M0 .. M7 that keep replays identical:
- a thread census;
- `Plain<T>` / `!Sync` as the safety net;
- an instrumented check of reads that leave their field.

Overlaps race B7, effects B4, menus D, physics B8. Depends on W4 and W5.

## Suggested order

1. W9 and W3 (cheap, mechanical, make everything after easier to read), and W1 if the
   owner wants cross-machine identical replays.
2. W7 and W8 (the biggest readability wins: tables as data, the duplicated menu tables).
3. W4, then W5, then W6, module by module (each makes the next smaller).
4. W2 (by the end of the project).
5. W10.

Each area file ends with its own batch order and the conflicts between its batches.
