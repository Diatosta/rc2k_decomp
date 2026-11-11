# Review: physics, math, sound, audio, input

Area: `src/{physics,math,sound,audio,input}` (about 49k lines, 427 files) and the matching
`rc2k_core/src/*` (about 17k lines, 85 files). Read-only review, 2026-11-10. Sampled, not
read in full: the biggest files (`damage_gauges`, `drive_wheels_step`, `x87_float`, `x87`),
all headers and stores, greps for the rest. Counts are `grep -rc` over the area unless noted.

## Summary

- `rc2k_core` in this area is in good shape: typed, small, plain `f32`/enums, few unwraps (all in tests bar
  `sound/cars.rs:55`). It is the target form. Nearly all the debt is in `src/`, where the
  same logic is still wrapped in "bits in a cell, x87 helper, bits out" code.
- Counts in `src/` (physics / math / sound / audio / input):
  `.bits()` 1362/81/101/0/10; `set_bits` 652/15/29/0/0; `flt_/dword_/unk_/byte_/word_` names 1425/36/152/9/150;
  six-digit hex addresses 3114/38/257/597/171; `&Race` parameters 159 functions, `&Scene` 172
  (physics); `_in_game` twins 9 (sound) / 29 (audio) / 4 (input); `_plain` twins 19 / 5;
  Regs-shaped text (eax/ebx/Regs) 198/17/37/125/63; test-only x87 reference code 19,009 lines in
  58 `cfg(all(test, target_arch="x86"))` blocks (bulk: `math/x87_reference_{a,b,c}.rs`
  6,069 lines, `math/x87.rs` 1,402, the `asm_reference` modules of `damage_gauges` etc.).
- Biggest single items: (1) the damage model (`damage_gauges` 3,482 + `damage_factors` 1,043 +
  `damage_wear` 791 + `rate_damage` 304 lines) is an x87 transcription (`t0..t92`, `flt_75b3c8`)
  that should be one plain `rc2k_core` module; (2) `math/x87*.rs` (187 + 69 + 22 call-site-shaped
  helpers) should be dissolved into the callers as plain `f64` expressions; (3) the sound
  command queue is still a ring of 7-register records decoded back into an enum; (4) the
  image-layout stores (`sets_store.rs` 3,346 lines of byte blobs and tagged handles,
  `audio/store.rs` 2,474, `input/store.rs`, `sound/store.rs`).
- Two correctness/determinism findings worth doing before anything cosmetic:
  `rcpss`/`rsqrtss` (hardware-dependent, `src/arch.rs:32-52`, used by `math/x87.rs:100`,
  `math/x87_run.rs:66`, `normalise_vector.rs`) and libm `sin/cos/atan2` (no `libm` crate in either
  `Cargo.toml`). Held replays are not bit-identical across CPUs/OSes today; see B1.

## The x87 question: where plain floats are bit-identical, where not

How the emulation works (`math/x87_float.rs`, `math/x87.rs:425-480`): `X(f64)`; `ld(bits)` =
`f64::from(f32)`; `st(x)` = `x as f32` under round-to-nearest; per-thread emulated control word
(`CONTROL`, default 0x027F). Every `X` op first asks `rounds_to_nearest()` and, if not nearest,
corrects the f64 result by an error-free transformation (two-sum, fma, remainder).

Facts that decide everything:
1. Production code leaves nearest rounding in exactly three places: `effects/pixels.rs:280`
   (chop), `effects/spans.rs:163-239` (chop / own), `effects/lines.rs:271` (`finit`, 0x037F =
   64-bit precision, NOT modelled: "within the last bit"). Everything in physics, sound, input,
   math and audio runs at 0x027F: 53-bit precision, round to nearest. `workers.rs:270-275`
   copies the control word to worker threads; `drive/tyre_turned_quad.rs` and `race/car_shadow.rs:121`
   only read/branch on it (they belong to other reviewers).
2. So in this area `X` == `f64`, `ld` == `as f64`, `st` == `as f32`, bit for bit (NaN: a signalling
   NaN is quieted by `cvtss2sd`/`cvtsd2ss`, which is what `rc2k_core::physics::x87::loaded`
   emulates; the x87 and SSE agree). The directed-rounding code (`directed`, `add_directed`,
   `mul_directed`, `div_directed`, `quarter_pis`, the `Rounding::*` arms of `st`, `sqrt`, `rint`)
   is dead for this area and live only for the effects.
3. It is NOT `f32` arithmetic. A chain `a*b + c` with f32 inputs is computed in f64 and rounded
   once to f32 at the store. f32 mul is exact in f64 (24+24 <= 53 bits), but the following add
   rounds twice (f64, then f32), which differs from a single f32 rounding in rare cases. Replacing
   by `f32` ops (or `mul_add`) would change replays. The right plain form is `f64` locals and
   `as f32` at each original store: e.g. `(f64::from(rate) * f64::from(step) + f64::from(g)) as f32`.
   Keep the original's operation order (`a + (b + c)` etc.) and the places where the original
   stored and reloaded (`fstp` then `fld` = a rounding to f32).
4. Cases that cannot be plain `f64`/`f32` and must stay a named, tested function:
   - `fldpi`-based operations (`times_pi`, `over_pi`, `pi_over`, `atan2_over_pi`): the x87's pi is 64-bit
     (0xC90FDAA22168C235), not `f64::PI`; results differ in the last bit for some inputs. Keep
     them in one `pi64` module (exact via the error terms already written).
   - `fprem` is `%` (exact), `fsqrt` is `f64::sqrt` (correctly rounded): plain.
   - `fsin/fcos/fpatan`: the original is x87, result "within an ulp" of libm. These are already NOT
     bit-identical to the 1999 game; they are libm's. So the constraint is determinism of this build
     across platforms, see B1.
   - `rcpss`/`rsqrtss` approximations: not plain, see B1.
   - 80-bit carries (`[u8; 10]`, `f80`, `from_f80`, `length_80`, `scale_by_80`): at 53-bit precision the
     boundary conversion is exact, so replace by an `f64` in the signatures (`[u32;3] -> f64`).
   - `finit`/64-bit-precision regions in `effects/lines.rs`: not provable plain; out of this area.
5. Proof method for each replacement, in order of strength: (a) exhaustive over the input space where
   it is small (`to_int`, `loaded`, 16-bit tables); (b) the existing `x87_reference_*` tests run once
   more on an x86 box against the new plain function with at least 10^7 random + edge-value inputs
   (0, -0, subnormals, inf, NaN with payloads, 2^24+1, values straddling a rounding), before the
   reference code is deleted; (c) a replay hash: run the held replays (the project's `remote.py`
   / frame hashes) and require identical per-frame state hashes; (d) freeze: write the (inputs,
   expected bits) pairs from (b) into a `tests/golden/*.bin` or an in-source `const` table, so the
   assembly can be deleted (B13).

## Batches (ordered by value for effort; files per batch are disjoint)

### B1. Determinism of the non-IEEE float operations (S-M, risk: changes replays across CPUs, not on the owner's)
Files: `src/arch.rs`, `src/math/x87.rs` (`rcpss_times` :100), `src/math/x87_run.rs` (:66, `face_shares`),
`src/math/normalise_vector.rs`, `src/math/x87_float.rs` (sin/cos/atan2), the 5 other `.sin()/.cos()` sites
(`physics/update_camera.rs` 2, `physics/wall_edge_contact.rs`, `physics/face_contact.rs`,
`sound/play_placed_sound.rs`, `input/update_car_effects.rs`; `rc2k_core/src/math/vector.rs:114`,
`rc2k_core/src/input/feedback.rs:75`, `rc2k_core/src/race/sound_tables.rs:44`).
1. `arch::rcpss/rsqrtss` use the hardware instruction on x86 (Intel and AMD give different bits; the
   handheld and Mac builds use exact `1.0/a`). So the same replay can differ per CPU. Proposal: a
   software `rcp_estimate(f32)` / `rsqrt_estimate(f32)` in `rc2k_core::math::estimates`, a 2048-entry
   table indexed by the top mantissa bits reproducing the Intel results (the game was developed on
   Intel; pick Intel and state it). Proof: exhaustive over all 2^32 f32 inputs against `_mm_rcp_ss` on an
   Intel CPU once (a few seconds), then freeze the table plus a CRC of all outputs as the test. Say
   clearly it changes results on AMD/ARM/handheld (it makes them the same as Intel's).
2. `f64::sin/cos/atan2` call the platform libm (glibc, UCRT, Apple, musl differ). Proposal: add the
   `libm` crate (pure Rust, deterministic) and route every `sin/cos/atan2/tan` of the game through one
   `rc2k_core::math::trig` module. Replays then equal across platforms but differ in rare last bits
   from today's glibc results; so re-record the held replay hashes once and note it. Mechanical.
Dependencies: none. Do before B2/B3 so the later hashes are taken after the change.

### B2. Dissolve `math::x87`, `x87_run`, `x87_load` into plain f64 at the callers (L, risk low with proof (b)+(c), mostly mechanical)
Files: `src/math/x87.rs` (187 pub fns), `x87_run.rs` (69), `x87_load.rs` (22) and their ~950 call sites
(`effects/` 97 files, `physics/` 89, `scene/` 28, `drive/` 14, `race/` 24, `sound/` 8, `hud/` 4).
Problem: the module is named by nothing ("math::x87") yet is a bag of call-site-shaped helpers with
`u32` bits in and out: `eased_roll(i,a,b,v,c,m,h)`, `sun_direction(a,k,s,b,m)`, `abs_blend_compare`, `drifted`,
`clip_lerp3`, ... 93 of them have exactly 2 users and 80 have 3 (counted outside tests), 1 has 1; half a
dozen (`mul`, `add`, `to_int`, `from_int`, `loaded`) are used hundreds of times. Names say how (the
instruction), not what (the quantity).
Proposal, in three steps:
1. (S) Replace the trivially expressible helpers by typed expressions: `x87::mul(a,b)` with floats
   -> `(f64::from(a) * f64::from(b)) as f32` via a tiny `rc2k_core::math::wide` module (`pub fn mul(a:f32,b:f32)->f32`,
   `fma_like(a,b,c)`) documented as "evaluated in f64, rounded once to f32, as the original's x87 did". Delete `X`, `ld`, `st`
   for this area.
2. (M) Move each multi-step helper (2 users) next to its caller as a private `fn` with a real name
   and `f32` parameters; ones shared by two modules go in `rc2k_core::math::<topic>` (matrix, spline, camera).
3. (M) Remove the whole directed-rounding machinery from `x87_float.rs` once effects (other reviewer)
   no longer uses it, or keep it only as `effects::rounding` with chop-only (`Rounding::Chop` is the
   only mode used: 0xC00). Control word thread-local (`CONTROL`, `set_control_word`, `workers.rs:270`)
   then becomes a parameter of the few chop functions.
Constraints: keep operation order and store points (fact 3). Do NOT turn into `f32` ops. Do NOT replace
`times_pi/over_pi/pi_over/atan2_over_pi` (fact 4). 80-bit `[u8;10]` signatures become `f64`.
Order: B3-B5 first or together, since they own the main users. Not mechanical in step 2 (naming);
steps 1 and 3 mostly mechanical.
Note: `math/x87.rs:1-3` header says "No x87 instruction runs outside the tests"; the module name x87
then lies. Rename target: `math::wide` (f64-evaluated float steps).

### B3. The damage model into `rc2k_core` (L, risk medium, proof by golden I/O)
Files: `src/physics/damage_gauges.rs` (3,482 lines; 242 `flt_75b../dword_75b..` names, 204
`race()` uses in the test part, `t0..t92` temporaries), `damage_factors.rs` (1,043), `damage_wear.rs` (791),
`rate_damage.rs` (304), `damage_step`, `store.rs` (`DamageModel`, 129 `flt_/dword_` fields, many documented
"not worked out"), `rc2k_core/src/physics/damage_notices.rs`.
Problems: (a) the logic is the x87 code transcribed 1:1 (`add_product`, `add_scaled_triple`, `gauge_totals`:
92 temporaries for sums of 6-9 gauges); (b) `steps(race)` repeats the same 4-line `raise(..., add_product(...))`
block about 40 times (lines 48-400; the table of gauge, rate index, factor is the data); (c) fields named by
address: `flt_75b38c`, `flt_75b3b4`, `dword_75b3b8` (hit direction +-1 as 0xFFFFFFFF, compared with `== 0xFFFFFFFF`),
`dword_75b3c4` (hit side 1..4), `dword_75b3bc` (gear -1..7); (d) takes the whole `&Race` (and `Shared`) where it uses
`race.physics.damage` and `race.shared.car`; (e) the "gear -1..7" refusal is repeated 10 times (`BadInput("a gear outside g_gearRatios")`).
Proposal: new `rc2k_core::physics::damage` with
`struct DamageModel { /* f32/i32 fields with real names */ }`, `struct Gauges { engine, exhaust, cooling, clutch, differential,
electrics, lights: [f32;6], steering, suspension: [f32;4], turbo, brakes: [f32;2], gears: [f32;9], ... }`,
`enum HitSide { Front(Left|Right), Rear(..) }` (1..4), `enum HitDirection { Up, Down }`, `struct Gear(i8)` -1..=7 with `Gear::new`
fallible once at the boundary, so the repeated refusal disappears.
`pub fn step(model:&mut DamageModel, g:&mut Gauges, inputs:&DamageInputs)`. Express the rate table as
`const WEAR: &[Wear{gauge: GaugeId, rate: usize, by: Share}]` and loop. The `src/` side only builds
`DamageInputs` and copies results back (`bridge`), per project goal "explicit inputs and outputs".
Naming work: derive names from what each figure is (the file headers already say a few: speed share,
brake share, engine speed share, wear step); the "not worked out" ones stay `unknown_75b3xx` until named,
listed in a TODO table rather than hidden.
Proof: before the rewrite, add a recording harness that runs the existing `steps` over held replays and writes
(inputs-bits, outputs-bits) for every call into a golden file (the test then asserts the new code
reproduces it); compute in f64 and store `as f32` exactly where `st` is today (fact 3). Then delete the
`asm_reference` modules (lines 761+ of `damage_gauges.rs`).
Mechanical: partly (the table conversion by an agent with golden tests as the safety net).

### B4. Take the needed fields, not the whole `Race`/`Scene` (M, risk low, mechanical per function)
Files: `src/physics/*.rs`: 159 `fn ...(race: &Race` and 172 `scene: &crate::scene::owner::Scene` parameters;
`drive_wheels_step.rs` (1,385 lines, 30 helpers each taking `race` or `shared`+`scene`), `drive_grip_forces.rs`
(917), `wheel_surfaces.rs` (968), `contact_force.rs` (590), `update_camera.rs` (698), `car_physics_step.rs`.
Problem: any function can read any of the race's state; the reader must read the body to know what a step touches
(e.g. `steps(race, scene)` in `drive_wheels_step.rs:83` is 200 lines using `race.shared.car`, `race.course.numbers.zero`,
`race.physics.*`, scene tables). This is the biggest obstacle to the "explicit inputs and outputs" goal in this area.
Proposal: for each public function define the struct of borrowed pieces it needs:
`struct WheelsStep<'a> { car: &'a CarBlock, tables: &'a PhysicsTables, grips: &'a [Float;5], zero: f32, ... }`
built once in `car_physics_step` (the single place that knows the `Race`). Helpers take `&CarBlock`/`&PhysicsTables`
only. Start with the leaf functions (they have 1-3 fields) and work up. Also replace `race.course.numbers.zero.bits()`
(a stored constant 0.0 read through the race! grep `numbers.zero` count) by the literal `0.0`.
Cells: while `Float` cells exist these take `&Float`, later `&mut f32` (batch B6).
Dependency: do after B3 for the damage files; files under F7 are not touched here (scene drawing is elsewhere), but
`scene: &Scene` params cross into `src/scene/` types: only change the physics side.
Mechanical: yes for leaf functions (agent per file), reviewer for the struct designs.

### B5. Remove hook-era shapes: `Result<_, Refused>` that cannot fail, register names, `_plain`/`_in_game` twins (M, risk low)
Locations:
- `math/turn_point_xz_ebx.rs`, `turn_point_xz_ecx.rs`, `turn_point_xz.rs`, `turn_float_point_xz.rs`, `turn_scratch_xz.rs`
  (five variants of one rotation differing in which register held x; callers `physics/place_camera.rs:19,111,127`).
  Proposal: one `turn_xz(point:[i32;3], angle) -> [i32;3]` in `rc2k_core::math::geometry` (it already has `turn_xz(f32,..)`)
  and delete the four. Includes the `_plain` twins (`turn_scratch_xz_plain`, `turn_float_point_xz_plain`); the "exact"
  vs "plain" distinction (`place_camera.rs:19` "exact or plain") dies with the x87 emulation if both equal in nearest mode;
  prove with the existing reference tests, then replay hash.
- `physics/car_sounds.rs:62,89` (`place_plain`, `cue_plain`), `physics/sound_bands.rs:54,61` (`band_at_plain`,
  `band_value_plain`) -> drop the `_plain` suffix once the non-plain twin is gone; and `sound_band_1..4.rs` (four files) to one
  `sound_band(n)`.
- `_in_game` twins (sound 9, audio 29, input 4): e.g. `sound/play_menu_sound.rs:50`, `sound/step_car_stream.rs:61`,
  `sound/load_race_sounds.rs:158`, `sound/play_placed_sound.rs:110`, `sound/stop_cd_track.rs:29`, `audio/buffer_play.rs:38`,
  `audio/new_voice.rs:11`, `audio/sound_device.rs:59,64`, `audio/mix_channels.rs:11`, `audio/write_mix.rs:14`, `input/input_tick.rs:52`,
  `input/release_device.rs:40`. Pattern: `X_in_game(state, ...)` = the pure `rc2k_core` logic applied to the game stores. They are the
  bridge, not twins; rename to `apply_*`/drop the suffix, and make the pure part take its inputs, the bridge nothing else. Check each: where
  `_in_game` just forwards, inline it.
- Register-text headers: input/axes.rs (`Moved { carry, eax }` :29-33, a return shaped like the CPU's flags: make it
  `Option<AxisMove>` / `enum AxisMoved { Rest, Moved{axis, up}, None{last_diff} }`); `input/device_effects.rs:5` (decoded
  `ecx bits 8..19 / edx bits 20..31`: a `struct EffectChange` with fields); audio/take_commands.rs:6-15, sound/queue.rs:1-12 (see B7).
  Total 63 `eax|ebx|Regs` lines in input, 125 in audio, 37 sound, 198 physics: most are comments; rewrite them as sentences.
- Headers claiming old facts: "Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers above are the
  original's" in `physics/test_drive_run.rs:19`, `sound/play_tune_2.rs`, etc.: remove the register paragraph, keep the sentence of what it does.
- `physics/test_drive_run.rs`: 5000 steps of the engine/gearbox to measure acceleration figures at car load (see
  `src/physics/test_drive*`). If it runs once per car selection, cache per car model; if per frame, a bug. Check (S).
Mechanical: yes (agents per module). Avoid `place_camera.rs` conflicts with B4.

### B6. `Float`/`Dword` cells that are really plain fields; `bits()` and `get()!=0` flags (L, staged)
Counts: `.bits()` 1,362 (physics) + 81 + 101 + 10; `set_bits` 652 + 15 + 29; `get() != 0|== 0` 111 sites over the area
(e.g. `physics/damage_factors.rs:78 all_wheels_off.get() == 0`, `bump_cars.rs:101`, `camera_ground.rs:27 ground_found.get()==0`).
Problem: `session::Float` is an `AtomicU32` with `write_watch` hooks (`session.rs:111-137`) read through `.bits()` so the x87 helpers
can take `u32`. The atomic is only needed for cells the audio thread or worker threads share. For physics state touched only on the
game thread (the whole `Car`/`DamageModel`/`PhysicsTables`/`Ground*` stores), a plain `f32` field is simpler and removes the write-watch
check on every access (also a speed gain, in the sense of F7). Booleans are `Dword` 0/1 (`ground_found`, `all_wheels_off`, `no_place`,
`start_in_gear`): make them `bool`.
Proposal: (1) after B2 removes `ld/st`, `Cell32::bits()` loses its main caller; (2) convert store by store: `physics::store`, `car_store`,
`tables_store`, `ground_store`, `camera_store`, `contact_store`, `gear_store`, `pace_store`, `split_tables_store`, `ground_probe_store`
(10 files, 4.4k lines) from `Dword/Float` to `u32/i32/f32/bool` plain struct fields; the `#[derive(Cells)]` and `#[repr(C)]` image layout
stay only where `load_car`/`store_car` still copy by offset (they should become `Clone` of a typed `CarBlock`; see B9).
Flags: replace `Dword` flag words with `bitflags`-style consts or an enum per field (`car_flags` bit 0x10 "steering trim negated" ->
`trim_negated: bool`). Use `Gear`, `HitSide` newtypes as in B3.
Risk: medium (each store is read from other modules: scene, hud, race). Do one store at a time; compile-driven; replays identical because values
do not change. Not safe for `fn()` shared across threads: keep atomics for the audio-thread-shared stores (`audio::AudioState`, `sound` queue).
Mechanical: yes per store with compiler guidance; names for the `unknown_*` stay.

### B7. Sound: the register-shaped command queue and image-layout set tables (M-L, risk medium)
Files: `src/sound/queue.rs` (`send_command(command, registers:[u32;7])`, 0x400 x 0x20-byte records), `senders.rs`, `src/audio/take_commands.rs`,
`stream_commands.rs`, `rc2k_core/src/audio/commands.rs` (`Command` enum + `decode`/`put`/`record`), `sound/play_tune_{2,3,4,6,7,8,9}.rs` (seven
7-line files: `play_tune(N)`), `sound/send_command_8.rs`, `sound/send_end_of_turn.rs`, `audio/store.rs` (`CommandQueue`), `sound/queue.rs:52 hash_if_asked`.
Problem: the game packs a typed `Command` into eax..ebp, writes it to a ring in "game memory", and the audio thread decodes it back (`decode`).
Both ends are Rust now. The 7 u32 registers `[eax,ebx,ecx,edx,esi,edi,ebp]` are not a meaningful interface; `Command::Stream{path,volume,action}` already exists
on the receiving side.
Proposal: `static QUEUE: Mutex<VecDeque<Command>>` (or `std::sync::mpsc`) in `sound::owner`; senders build `Command` values directly
(`Command::Music{tune, next, volume, action}`); `audio::take_commands` drains. Keep the 0x400 bound (`Put::Full` drops) and the "no audio -> count cleared"
behaviour. Delete `put`, `record`, `decode`'s register half, `command_count/command_write/command_read` cells and the `CommandQueue` store (:1,319
lines of `sound/store.rs` partially). Replace the seven `play_tune_N` files by `play_tune(Tune::N)` with `enum Tune` named by what they are
(menu tune, race tune..., from the sound sets). `hash_if_asked` (debug env hash with `DefaultHasher`, `RC2K_SOUND_HASH`) is a test aid inside
production: move behind the debug overlay/feature or delete after the change.
Risk: audio ordering; the queue is FIFO either way; add a unit test with a recorded command sequence (golden). The ordering between commands from
several threads is already by the atomic add; a `Mutex` keeps FIFO.
Dependencies: none (own files). Mechanical: no (design); the sender call sites (~30 `send_command(...)`) are.

### B8. Sound/audio/input stores: image layout that serves only the old memory model (L, risk low-medium)
Files and sizes: `src/sound/sets_store.rs` (3,346 lines: `#[repr(C, align(16))]` structs of `Bytes<N>` with `with_runs` byte literals,
`_lead`/`_tail` padding, tagged handles `RACE_LIST_TAG 0xC400_0000`, `CUE_TAG`, `SURFACE_TAG`, `KIND_TAG` with `offset_ref(tag, ida_address, start, len)` asserts),
`sound/store.rs` (1,319), `audio/store.rs` (2,474; 27 `_gap_` buffers "ids of property sets nobody asks for, A3D registry keys, unused floats"),
`audio/names_store.rs`, `input/store.rs` (617), `input/store_data.rs` (209), `input/guids.rs` (DirectInput GUID table by image address), `input/layouts.rs`,
`input/types.rs` (626: `Controller`, `InputDevice`, `DiJoyState` `#[repr(C)]` at IDA offsets, `field!` offsets).
Problems: the lists of sound numbers ending in -1 are stored as raw LE bytes and read back by address handle (`offset_in`, "the list runs on in its piece as far
as the sounds read"); the headers say the thing exists "because the audio thread reads by address" which is no longer true (phase 5 done; `rc2k_core` has typed `SoundList`).
Proposal: (1) `sets_store`: parse the sets once into `static RACE_SETS: [&[u16]; N]`/`Vec<SoundId>` slices (`const` arrays of `SoundId(u16)`), a `enum CarSoundKind`,
`enum SurfaceSet`; the tag handles disappear. Lists with the -1 terminator become slices. The tables of pointers become `&'static [&'static [SoundId]]`. Keep one test
that the new constants equal the old bytes (generated once from `with_runs` data) and then delete the bytes. (2) `audio/store.rs`: drop the `_gap_` fields (27) and the
EAX/A3D property GUID blobs nobody asks for (verify with grep that none is read: `grep -rn "_gap_"` shows only constructor lines), keep real state in typed
structs (`SoundMode`, `Volumes`, `VoicePool`); `Voice`, `MixerChannel`, `SampleRecord` already live in core, so the 0x200 voices of 0x70 bytes should be `Vec<Voice>`.
(3) `input/types.rs` and `guids.rs`: DirectInput object structs exist for the COM stand-in (see B9).
Order: after B7. Mechanical: partly (sets_store conversion can be generated by script from the existing bytes).
Risk: the sets are read by many (`sound::*`); compile-guided. Replays unaffected (sound does not feed physics).

### B9. DirectInput emulation: `input/di_objects.rs` and friends (L, risk medium; needs owner decision)
Files: `input/di_objects.rs` (838), `di_harness.rs`, `types.rs` (626), `guids.rs`, `acquire_device.rs`, `list_joysticks.rs`, `joystick_found.rs`,
`axis_found.rs`, `begin_force.rs`, `end_force.rs`, `effect_found.rs`, `make_effect.rs`, `effect_objects.rs`, `release_device.rs`, `poll_device.rs`, `src/platform_input.rs` (577;
SDL backend; other reviewer).
Problem: the game's code calls an in-process DirectInput 5 stand-in with GUIDs, `DIPROP`s, `DIERR_` codes, `QueryInterface` and reference counts
(`di_objects.rs:1-20`), whose `Backend` trait is then implemented over SDL. The project rule says every input ends up handled by SDL, no emulation.
Proposal: replace the stand-in by a trait `Joysticks` (`devices() -> Vec<JoystickInfo>`, `open`, `state() -> JoyState{axes:[i32;6], buttons, hats}`,
`range`, `dead_zone`, `rumble(effect: ForceEffect)`) with typed `ForceEffect` (the game's constant, periodic, condition, ramp kinds); the game's 10 device
functions become methods; GUID tables go. Keep deadzone/saturation/range maths (the games' `Axis`) in core with tests.
Needs an owner decision because this changes the force-feedback path (hardware-tested only by the owner). Mechanical: no.

### B10. Naming sweep (S-M per file, mechanical with a rename map; risk none)
- Fields `flt_75b3xx/dword_75b3xx/unk_71axxx` (1,425 matches in physics; the worst: `store.rs` 129, `damage_gauges` 242, `rate_damage` 42, `damage_wear` 39,
  `damage_factors` 35, `ground_store` 16, `sound/store.rs` 14) -> semantic names; where the name is not known write `unknown_75b3xx` with its address in a doc comment only
  (not in the identifier); the current doc comments saying "not worked out" (about 40 in `store.rs`) become a single `TODO(name)` list in `notes/`.
- `car_store.rs` has 11 `unknown_*` + 8 in `ground_store`, 22 in `tables_store`, 6 in `sets_store`: same.
- File names that are addresses or counters: `physics/take_view_a.rs`, `take_view_b.rs`, `copy_car_word_8.rs`, `set_flag_51659c.rs`, `sound/send_command_8.rs`,
  `sound/play_tune_*`, `sound_band_1..4.rs`, `math/turn_point_xz_ebx.rs/_ecx.rs`, `math/x87_reference_a/b/c.rs`.
- Module `math::x87` -> `math::wide` (see B2). `physics::flow` (fn `g(x)`, `p(x,v)` aliases for `.bits()`/`.set_bits()`: remove, they save no typing),
  `physics::cells` -> after B6.
- Inconsistent naming of one thing: `numbers.zero` (the constant 0.0, a field), `fixed_point_scale`, `flt_5C5D98` ("scale 256"/"1/256") appear under several names; grep once
  `numbers\.\|fixed_point_scale` and unify as `const`s in core.
Do last in each file's life (after B3/B4/B6 touch it) to avoid conflicts; for files untouched by other batches do it now. Mechanical: yes with a map; the map itself needs a human for the
unknown ones.

### B11. Structure: very long files and functions (M, risk low)
- `physics/damage_gauges.rs` 3,482 (B3), `drive_wheels_step.rs` 1,385: `steps()` is a single function of ~200 lines (:83-286) mixing slip, clutch, brakes, handbrake, drag; split into
  `slip_shares`, `drive_force`, `brake_step`, `side_forces` called in order, each with a typed input struct (B4). `update_camera.rs` 698, `wheel_surfaces.rs` 968, `drive_grip_forces.rs` 917.
- `audio/store.rs` (2,474) and `sound/sets_store.rs` (B8); `input/di_objects.rs` (B9).
- `math/x87.rs` (1,402) + `x87_reference_c.rs` (2,620): see B2/B13.
- Test-only items in production files: `physics/cells.rs` (helpers named `set_changed`, `set_values_changed`: "write only if different", needed because a copy
  of the original wrote only changed words; check whether still needed or just a write-watch artefact), `physics/test_drive_run.rs` name ("test" = the car test drive feature; rename `acceleration_test`).
  Use `grep -rn "cfg(test)" ` per file; ~58 `cfg(all(test, target_arch="x86"))` blocks sit in the same files as production code: move each into `<file>/tests.rs` or `tests/` so reading a
  production file does not mean skipping 40% of it (`damage_gauges.rs:761-3482` is 78% test).
- `pub` that could be private: `physics::flow` helpers (`g`, `p`), `sound::queue` consts; run `cargo clippy` with `unreachable_pub` once on these modules (agent task).

### B12. Errors: refusals that cannot happen, panics on impossible state (S-M, risk low)
- ~99 `BadInput(` in physics, 74 sound, 77 audio, 64 input, 11 math. Common text list (physics): "a gear outside g_gearRatios (-1 ..= 7)" x8, "the gear is not -1 .. 7" x2,
  "no course faces or points" x2, "a part past 0xFF" x2, "a null car block" x2, "more than 0x40 probe points" x3, "the kind is not 0 or 1". Each of these is a validation of
  a value that was an unchecked table index in the original; once the value has a type (`Gear`, `PartIndex(u8)`, `ProbeCount`, `PartKind`), the check moves to the single
  constructor and the `Result` threading through `drive_wheels_step`, `revs_from_wheels`, `damage_*` etc. goes (many functions return `Result<(),Refused>` only for that).
  Process: for each message find the producer; if the value comes from a table that Rust writes within range, replace by a type or `debug_assert!`.
- Fatal path in non-Result functions: `crate::fatal::raise::end("a gear's wear gauge", BadInput(..))` inside `gear_gauge` (`damage_gauges.rs:24-30`), `kind()` :188: ends the program
  from deep inside logic; with `Gear` newtype impossible.
- Unwraps: only `rc2k_core/src/sound/cars.rs:55 try_into().unwrap()` (a slice-to-array that is length-checked just above?), 9 in `src/math` (tests?), `sound` 9, `audio` 11, `input` 8:
  check each for production reachability; replace by `let [..] = slice else`.
Mechanical: partly; coordinate with B3/B6 which create the types.

### B13. Tests that compare to the original's assembly or image (M-L, risk none to behaviour, deletes the original's code from the repo)
Inventory: `math/x87_reference_a.rs` (1,704), `_b` (1,745), `_c` (2,620) = 6,069 lines of inline-assembly reference versions run on x86 only;
`math/x87.rs` `x87!` macro and the `#[cfg(all(test,target_arch="x86"))]` `row_times_column`, `add_into`; `asm_reference` modules in
`damage_gauges.rs:761+`, `drive_wheels_step.rs:824+`, `nearest_on_segment.rs:109+`, ~50 more in physics (58 cfg blocks in total, 19,009 lines in files that contain them);
`first_values_are_the_games` / `*_start_as_the_games` image checks (30 occurrences: `store.rs` of each module, `next_random.rs`, `normalise_vector.rs`, `input/guids.rs`, `layouts.rs`,
`audio/names_store.rs`, `sound/sets_store.rs`) which read `data/ral_image.bin`.
How to freeze each (so the repo can lose the original's code and data):
1. Generator step (one time, on an x86 box, keep the generator in `tools/` outside the crate or delete after use): for every reference function, produce N (default 2000) input vectors:
   edge values (0, -0, +-1, subnormal, max, inf, NaN with payload, values at 2^24 boundaries) + seeded random; run the assembly; write `(inputs, outputs)` as a
   `const CASES: &[(...)]` table in `<module>/tests/golden_<fn>.rs` or a binary file under `rc2k_core/tests/data/`. Size: ~190 helpers x 2000 x 16 bytes = 6 MB: use 500 cases + the edge set => ~1.5 MB; or one hash per function (`crc32` of all outputs for a fixed seed in a deterministic xorshift generator that the test re-runs) - 190 hashes, a few KB, and the test needs no data: it regenerates inputs from the seed, runs the new plain function and compares the CRC. Preferred.
2. Exhaustive where possible: `to_int`, `loaded`, 16-bit tables, rcpss (B1).
3. Sequence-state tests (`damage_gauges`, `drive_wheels_step`): freeze a few thousand real frames from held replays: state-in hash and state-out hash per step, as in B3.
4. Image checks (`start_as_the_games`): replace by `const` first values in source (the data is already the Rust source of the stores); once a store is rewritten (B6/B8) the check goes with it.
   The game's non-code data in `data/ral_image.bin` may stay for the owner's play; just not needed by tests.
5. Then delete `x87_reference_*` and the `x87!` macro; `cfg(target_arch="x86")` test blocks vanish, tests run on all platforms (today none of them run on the Linux x86_64 or handheld CI: they are skipped by cfg, so a wrong helper there would go unseen).
Order: B13 follows B2 per helper (freeze, then delete).

### B14. Comments (S, mechanical rewrite)
- Stale: `math/x87.rs:1-3`/`x87_float.rs:1-13` fine but claim live rounding modes; `math/x87.rs:429` "(phase 6, 2026-10-24)" dating style; `physics/damage_gauges.rs` `add_product` doc
  "An x87 run of the original ... `[rc + 0x31A320] = ...`" repeated 12 times with address expressions instead of a sentence ("raise `g` by `rate * step`");
  `sound/sets_store.rs:5-7` ("the audio thread reads by address"); headers with "Takes:" lists that are leftover register notes (`physics/point_hits_car.rs:14`), "Callers: (both `jb`)".
- Dense/cryptic: the `//!` header paragraphs listing 20 `dword_614...` names (`point_hits_car.rs:1-13`, `test_drive_run.rs:1-18`): replace by 3-4 lines of purpose (what the car test is, what is measured, what is output) and put the address trail in `notes/`.
- Missing: the reasons for tricky choices. `st` / "operation order and store points matter" (fact 3 above) deserves a doc comment on `rc2k_core::math::wide` so nobody "simplifies" `(a as f64 * b as f64 + c as f64) as f32` into f32 ops;
  `exact_angle_over_pi` explanation exists (good).
Mechanical: yes for removal of redundant address blocks; a human for the "what" sentences.

## Notes on `rc2k_core` in this area (smaller items)
- `rc2k_core/src/physics/x87.rs`: name and doc speak of the x87; contents are `loaded` (NaN quieting), `below`/`equal` (unordered = true), `hold_low/hold_high`, `fistp`. Rename `float_quirks.rs` and document in terms of the observable behaviour (unordered compares count as less / equal), as the 1999 code relied on it.
- `rc2k_core/src/physics/*` has 44 files / 7.7k lines with `// converted ...` provenance in headers; fine, but the `math::units` / `extended.rs` / `geometry.rs` pieces should absorb the `src/math/*` leftovers (B5).
- `rc2k_core/src/audio/voices.rs` (1,004 lines) mixes `Voice`, `MixerChannel`, `SampleRecord` and the voice allocation logic; split `records.rs` / `allocation.rs` (S).
- `rc2k_core/src/input/feedback.rs:75` and `math/vector.rs:114`, `race/sound_tables.rs:44` use libm trig directly (B1).
- Tests in `rc2k_core` are real unit tests and need no data; keep as the pattern for B13.

## Files touched by F7 / not to edit now
None of the files named above is on the F7 list (`src/workers.rs`, `race/view_scratch.rs`, `race/draw_race_scene.rs`, `scene_hand_off`, `view_window`, `render_cache`). Dependency to remember: `src/workers.rs:270-275`
copies the x87 control word onto worker threads (B2 step 3 removes the need: wait until F7 is done), and `physics::place_camera` / `scene::*` callers of `math::x87` (B2) cross into scene files.

## Suggested order
1. B1 (determinism; small; do first so later hashes mean something).
2. B7 (independent; big readability win in sound), B12 types (`Gear`, `PartIndex`) alongside B3.
3. B3 and B4 together for the physics files (they overlap in `damage_*`, `drive_*`; one agent per file group).
4. B2 (steps 1-2) once B3/B4 have removed the biggest users; then B13 (freeze + delete) per helper; then B2 step 3 after F7.
5. B6 (cells -> fields) store by store; B8 (stores) and B9 (DirectInput, owner decision).
6. B5, B10, B11, B14 as clean-up passes per module after the structural batches have touched the file.
Parallelism: B1, B7, B8-sound, B9, B5-input/audio can run at once (disjoint dirs); B3/B4/B6 share `physics/*` so run them serially or split by file group.
