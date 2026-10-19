//! The race sounds' turn (stream B, 2026-10-18): `sound::race_turn` (0x40D5F3) and the
//! trees under it, which work on the eight cars' sound objects (`g_raceSoundObjects`,
//! 0x4A8 bytes each), the shared spots' voices (`unk_5168C0`) and one request
//! (`unk_5124BC`, ten dwords, see `sound::play_placed_sound`) they send each sound with.
//! This file: their calls by place, the place being the callee's IDA address: in the game
//! the Rust versions, in the tests stand-ins that note each call with its registers (and
//! the request when esi points at it) and change nothing but eax for the three that give
//! a number; and copies of the originals whose calls (and jumps out) go to them.
//!
//! Shadow mode runs the originals that send sounds alone (a sound must not be sent
//! twice); those that only work out state run both, compared (`dispatch_turn`).

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, game_read, rva, shadow};
use crate::install::stand_ins::{set_handler, stand_in};
use crate::redirect::RedirectedCopy;
use crate::service::harness::{Region, capture, restore};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

pub type Part<'a> = &'a mut dyn FnMut(u32, &mut Regs) -> Result<(), Refused>;
/// The state a Rust version that sends nothing is compared on in shadow mode.
pub type Regions<'a> = &'a dyn Fn(&Regs) -> Vec<Region>;
/// A Rust version: the registers as the original leaves them.
pub type Rust = fn(&mut Regs, Part) -> Result<(), Refused>;

pub const PLAY: u32 = 0x409B36; // sound::play_sound
pub const STOP: u32 = 0x40F1D1; // sound::stop_sound
pub const PLACED: u32 = 0x40F11C; // sound::play_placed_sound
pub const RANDOM: u32 = 0x4018B4; // math::next_random
pub const BELOW: u32 = 0x4018A2; // math::random_below
pub const TICK: u32 = 0x411024; // system::tick_count
/// `sound::step_race_clock`: carry set, no turn due.
pub const STEP_CLOCK: u32 = 0x40F20E;
/// `input::update_effects`: the force feedback's requests (the game's side; DirectInput
/// gets them on the input thread).
pub const EFFECTS: u32 = 0x40F2C0;

/// unk_5124BC: the request the turn's sounds go with.
pub const REQUEST: usize = 0x5124BC;
pub const REQUEST_LEN: usize = 0x28;
/// dword_5124E8: the time of the sounds' last turn (`sound/race_clock.rs`).
pub const CLOCK: usize = 0x5124E8;
/// g_raceSoundObjects: eight pointers.
pub const OBJECTS: usize = 0x512A10;
/// dword_514F7C: the sound objects in use.
pub const COUNT: usize = 0x514F7C;
pub const OBJECT_LEN: usize = 0x4A8;
/// dword_5183CC: the scratch dword the x87 code goes through.
pub const SCRATCH: usize = 0x5183CC;

/// The calls in the game.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::sound::play_sound::play_sound_in_game;
    match place {
        PLAY => play_sound_in_game(r),
        STOP => {
            if let Some(with) = unsafe { crate::sound::stop_sound::stop_sound(r) }? {
                play_sound_in_game(&with);
            }
        }
        PLACED => unsafe {
            let mode = game_read(crate::sound::play_placed_sound::MODE_RVA);
            let samples = game_addr(crate::sound::play_placed_sound::SAMPLES_RVA);
            crate::sound::play_placed_sound::play_placed_sound(r, mode, samples, &mut |w| {
                play_sound_in_game(w)
            })?;
        },
        RANDOM => r.eax = unsafe { crate::math::next_random::next_random() },
        BELOW => r.eax = unsafe { crate::math::random_below::random_below(r.eax) }?,
        TICK => r.eax = crate::system::tick_count::tick_count(),
        EFFECTS => {
            crate::input::update_effects::update_effects(r, &mut crate::input::effects::game_part)?
        }
        0x40E59E => crate::sound::play_timed_slot::play_timed_slot_in(r, &mut game_part)?,
        0x40D8E2 => crate::sound::car_engine_state::car_engine_state_in(r, &mut game_part)?,
        0x40DD19 => crate::sound::revs_sound::revs_sound_in(r, &mut game_part)?,
        0x40DD96 => crate::sound::throttle_sound::throttle_sound_in(r, &mut game_part)?,
        0x40DE09 => crate::sound::speed_sound::speed_sound_in(r, &mut game_part)?,
        0x40DE9B => crate::sound::surface_sounds::surface_sounds_in(r, &mut game_part)?,
        0x40E068 => crate::sound::surface_sound::surface_sound_in(r, &mut game_part)?,
        0x40E0F6 => crate::sound::car_slot_sounds::car_slot_sounds_in(r, &mut game_part)?,
        0x40E182 => crate::sound::car_hit_sound::car_hit_sound_in(r, &mut game_part)?,
        0x40E1E5 => crate::sound::car_cue_sound::car_cue_sound_in(r, &mut game_part)?,
        0x40E24F => crate::sound::car_gear_sounds::car_gear_sounds_in(r, &mut game_part)?,
        0x40D893 => crate::sound::car_sounds::car_sounds_in(r, &mut game_part)?,
        0x40D75A => crate::sound::mark_car_voices::mark_car_voices_in(r, &mut game_part)?,
        0x40D7AB => crate::sound::car_sounds_all::car_sounds_all_in(r, &mut game_part)?,
        0x40ED13 => crate::sound::play_cue_queues::play_cue_queues_in(r, &mut game_part)?,
        0x40ED4A => crate::sound::play_cue_queue::play_cue_queue_in(r, &mut game_part)?,
        0x40EE32 => crate::sound::drop_cue::drop_cue_in(r, &mut game_part)?,
        0x40E2A9 => crate::sound::stage_sounds::stage_sounds_in(r, &mut game_part)?,
        0x40E611 => crate::sound::spot_sounds::spot_sounds_in(r, &mut game_part)?,
        0x40E642 => crate::sound::match_spot_voices::match_spot_voices_in(r, &mut game_part)?,
        0x40E6F4 => crate::sound::free_spot_voices::free_spot_voices_in(r, &mut game_part)?,
        0x40E733 => crate::sound::spot_voice_distances::spot_voice_distances_in(r, &mut game_part)?,
        0x40E7CA => {
            crate::sound::merge_spot_voices_all::merge_spot_voices_all_in(r, &mut game_part)?
        }
        0x40E809 => crate::sound::merge_spot_voices::merge_spot_voices_in(r, &mut game_part)?,
        0x4633DE => crate::math::nearest_on_segment::nearest_on_segment_in(r, &mut game_part)?,
        0x40E97D => crate::sound::thin_spot_voices_all::thin_spot_voices_all_in(r, &mut game_part)?,
        0x40E9F8 => crate::sound::thin_spot_voices::thin_spot_voices_in(r, &mut game_part)?,
        0x40EAF3 => {
            crate::sound::exclusive_spot_voices::exclusive_spot_voices_in(r, &mut game_part)?
        }
        0x40EB44 => crate::sound::repeat_spot_voices::repeat_spot_voices_in(r, &mut game_part)?,
        0x40EB9E => crate::sound::play_spot_voices::play_spot_voices_in(r, &mut game_part)?,
        0x409BE0 => crate::sound::send_end_of_turn::send_end_of_turn_in(r, &mut game_part)?,
        0x40F257 => crate::input::step_effects_clock::step_effects_clock_in(r, &mut game_part)?,
        0x40D5F3 => crate::sound::race_turn::race_turn_in(r, &mut game_part)?,
        0x421BCF => crate::sound::race_turn_if::race_turn_if_in(r, &mut game_part)?,
        // sound::send_command with command 0xA (`send_end_of_turn`'s one call).
        0x409C45 => crate::sound::send_end_of_turn::send_end_of_turn_in_game(r),
        STEP_CLOCK => {
            let due = unsafe {
                crate::sound::race_clock::step_race_clock(crate::system::tick_count::tick_count())
            };
            r.set_carry(!due);
            r.set_zero(false);
        }
        // The quit jump: the original, as it is (held for the owner's decision).
        crate::sound::race_turn::QUIT => crate::sound::race_turn::quit_in_game(r),
        _ => return Err(BadInput("a race sound call with no version").into()),
    }
    Ok(())
}

/// A part for the versions that call nothing.
pub fn no_part(_: u32, _: &mut Regs) -> Result<(), Refused> {
    Err(BadInput("a call where the original makes none").into())
}

// ---- the dispatch ----

/// The seven registers a call or a comparison looks at.
pub fn regs7(r: &Regs) -> [u32; 7] {
    [r.eax, r.ebx, r.ecx, r.edx, r.esi, r.edi, r.ebp]
}

/// The hooks' dispatch. With `regions` None (the function sends sounds) shadow mode runs
/// the original alone; with Some, both run on the same state, the registers and the
/// regions compared. Out of shadow mode, `rust` with the game's calls.
pub fn dispatch_turn(
    hook: &RawHook,
    r: &mut Regs,
    call_original: unsafe extern "C" fn(*mut Regs),
    rust: Rust,
    regions: Option<Regions>,
) {
    hook.count_call();
    if shadow() {
        let Some(regions) = regions else {
            unsafe { call_original(r) };
            return;
        };
        if !crate::service::harness::shadow_due(hook) {
            unsafe { call_original(r) };
            return;
        }
        let all = regions(r);
        let (entry, before) = (*r, capture(&all));
        let mut g = *r;
        // Nothing it calls sends a sound: the game's calls are the pure Rust versions.
        let mine = rust(&mut g, &mut game_part).map(|()| (regs7(&g), capture(&all)));
        restore(&all, &before);
        unsafe { call_original(r) };
        if let Ok(mine) = mine {
            let theirs = (regs7(r), capture(&all));
            if theirs != mine {
                let diff = theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b);
                hook.mismatch(|| {
                    format!(
                        "entry {:x?}: original {:x?}, rust {:x?}; memory first differs at {diff:x?}",
                        regs7(&entry),
                        theirs.0,
                        mine.0
                    )
                });
            }
        }
        return;
    }
    let mut g = *r;
    match rust(&mut g, &mut game_part) {
        Ok(()) => {
            *r = Regs {
                esp: r.esp,
                eflags: r.eflags,
                ..g
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(hook, e, r, call_original) },
    }
}

// ---- the tests' side ----

/// A call noted: the place, the registers, the request when esi pointed at it.
pub type Called = (u32, [u32; 7], Option<[u32; 10]>);

static CALLED: Mutex<Vec<Called>> = Mutex::new(Vec::new());
static PLACES: Mutex<Vec<u32>> = Mutex::new(Vec::new());
/// What the stand-ins for `next_random` and `random_below` draw from, and the clock the
/// one for `tick_count` gives.
static DRAW: AtomicU32 = AtomicU32::new(1);
pub static TEST_TICK: AtomicU32 = AtomicU32::new(0);
/// The carry the stand-in of `step_race_clock` gives: not 0, no turn due.
pub static TEST_NOT_DUE: AtomicU32 = AtomicU32::new(0);

fn request_at(at: u32) -> [u32; 10] {
    std::array::from_fn(|n| unsafe { (at as *const u32).add(n).read_unaligned() })
}

fn note(place: u32, r: &mut Regs) {
    let request = (r.esi == game_addr(rva(REQUEST)) as u32).then(|| request_at(r.esi));
    CALLED.lock().unwrap().push((place, regs7(r), request));
    let draw = || {
        let d = DRAW
            .load(Ordering::SeqCst)
            .wrapping_mul(0x41C6_4E6D)
            .wrapping_add(0x3039);
        DRAW.store(d, Ordering::SeqCst);
        d >> 1
    };
    match place {
        RANDOM => r.eax = draw(),
        BELOW => r.eax = if r.eax == 0 { 0 } else { draw() % r.eax },
        TICK => r.eax = TEST_TICK.load(Ordering::SeqCst),
        STEP_CLOCK => r.set_carry(TEST_NOT_DUE.load(Ordering::SeqCst) != 0),
        // Done as they are (a loop waits on the first, the result is read after the
        // second): pure, on what the test gave.
        0x40EE32 => {
            let _ = crate::sound::drop_cue::drop_cue_in(r, &mut no_part);
        }
        0x4633DE => {
            let _ = crate::math::nearest_on_segment::nearest_on_segment_in(r, &mut no_part);
        }
        _ => {}
    }
}

fn handler(n: u32, r: &mut Regs) {
    let place = PLACES
        .lock()
        .unwrap()
        .get(n as usize)
        .copied()
        .unwrap_or(u32::MAX);
    note(place, r);
}

pub fn test_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    note(place, r);
    Ok(())
}

pub fn take_called() -> Vec<Called> {
    std::mem::take(&mut *CALLED.lock().unwrap())
}

/// A copy of an original (IDA address `address`, `len` bytes, its first bytes `entry`),
/// its calls and jumps out (at `outs`, the opcodes' offsets) to `places` going to the
/// stand-ins, the others where they went.
pub fn copy_of(
    address: usize,
    len: usize,
    entry: &[u8],
    outs: &[usize],
    places: &[u32],
) -> Result<RedirectedCopy, &'static str> {
    set_handler(handler);
    *PLACES.lock().unwrap() = places.to_vec();
    let start = game_addr(rva(address));
    let stand = |target: usize| {
        places
            .iter()
            .position(|&p| game_addr(rva(p as usize)) == target)
            .map(|n| stand_in(n as u32))
    };
    unsafe {
        // The opcode at each offset: from `entry` where it covers it (a hooked function's
        // first bytes are now our jump), else from the function.
        let byte = |o: usize| match entry.get(o) {
            Some(&b) => b,
            None => (start as *const u8).add(o).read(),
        };
        let calls: Vec<usize> = outs.iter().copied().filter(|&o| byte(o) == 0xE8).collect();
        let copy = RedirectedCopy::new(start, len, entry, &calls, &|c| stand(game_addr(c)))?;
        for &o in outs {
            let at = (copy.addr() as *const u8).add(o);
            let n = match (at.read(), at.add(1).read()) {
                (0xE8, _) => continue,
                (0xE9, _) => 5,
                (0x0F, 0x80..=0x8F) => 6,
                _ => return Err("not a call or a jump out where the list says"),
            };
            let rel = (at.add(n - 4) as *const i32).read_unaligned();
            let target = (start + o + n).wrapping_add(rel as usize);
            copy.retarget(o + n - 4, stand(target).unwrap_or(target));
        }
        Ok(copy)
    }
}

/// One test case: `fill` then the original (`run`), `fill` then `rust` with the
/// stand-ins; the calls, the registers and `all` compared. With the Rust version
/// refusing, the original is not run (it might trap) and the case counts as passed only
/// if `refuses` says so.
#[allow(clippy::too_many_arguments)]
pub fn both(
    t: &mut SelfTest,
    what: &str,
    run: &dyn Fn(&mut Regs),
    entry: Regs,
    fill: &dyn Fn(),
    all: &[Region],
    rust: Rust,
    refuses: bool,
) {
    // Both sides start from the same state: what the first wrote in `all` is put back.
    let start = capture(all);
    fill();
    take_called();
    DRAW.store(7, Ordering::SeqCst);
    let mut g = entry;
    let ok = rust(&mut g, &mut test_part).is_ok();
    let mine = (regs7(&g), take_called(), capture(all));
    if !ok {
        t.check(refuses, || format!("{what}: the Rust version refused"));
        return;
    }
    restore(all, &start);
    fill();
    DRAW.store(7, Ordering::SeqCst);
    let mut ro = entry;
    run(&mut ro);
    let theirs = (regs7(&ro), take_called(), capture(all));
    t.check(!refuses && theirs == mine, || {
        let diff = theirs.2.iter().zip(&mine.2).position(|(a, b)| a != b);
        format!(
            "{what}: original {:x?} {:x?}; rust {:x?} {:x?}; memory first differs at {diff:x?}",
            theirs.0, theirs.1, mine.0, mine.1
        )
    });
}

/// `both` through a copy.
pub fn run_copy(copy: &RedirectedCopy) -> impl Fn(&mut Regs) + '_ {
    move |r: &mut Regs| unsafe { call_game(r, copy.addr()) }
}

/// A test sound object: 0x4A8 bytes of a pattern, with the fields given set.
pub fn test_object(seed: u32, fields: &[(usize, u32)]) -> Vec<u32> {
    let mut o: Vec<u32> = (0..OBJECT_LEN as u32 / 4)
        .map(|k| (k.wrapping_mul(0x9E37_79B9) ^ seed) & 0x3F7F_FFFF)
        .collect();
    for &(at, v) in fields {
        o[at / 4] = v;
    }
    o
}

/// The request and the scratch dword, which every one of these may write.
pub fn common_regions() -> Vec<Region> {
    vec![
        (game_addr(rva(REQUEST)) as u32, REQUEST_LEN),
        (game_addr(rva(SCRATCH)) as u32, 4),
    ]
}

/// Everything the turn's trees write but the sounds they send: the objects in use, the
/// request, the scratch dword and the variables between (for `guard`).
pub fn turn_regions() -> Vec<Region> {
    let a = |x: usize| game_addr(rva(x)) as u32;
    let mut all: Vec<Region> = (0..8usize)
        .map(|n| unsafe { game_read::<u32>(rva(OBJECTS + 4 * n)) })
        .filter(|&o| o != 0)
        .map(|o| (o, OBJECT_LEN))
        .collect();
    all.extend(common_regions());
    all.extend([
        // dword_514F80, dword_514F84, flt_514F88
        (a(0x514F80), 12),
        // flt_51638C .. flt_516398
        (a(0x51638C), 0x10),
        // dword_4F66F1, dword_4F66F5 (unaligned)
        (a(0x4F66F1), 8),
    ]);
    all
}

/// Bits of a float.
pub fn f(v: f32) -> u32 {
    v.to_bits()
}

/// The test of the three engine sounds (`sound::revs_sound` and the two after it): a copy
/// of the original at `address` whose calls and jumps out go to the stand-ins of
/// `stop_sound` and `play_placed_sound`, against `rust`, on an object and a sound kind
/// of the test's: +0 a player's and the computer's numbers, every state (+0x10C), +0xC
/// 0 and 1, and the values `extra` gives of other dwords, every combination.
pub fn test_car_sound(
    t: &mut SelfTest,
    address: usize,
    len: usize,
    entry: &[u8],
    outs: &[usize],
    rust: Rust,
    extra: &[(u32, &[u32])],
) {
    let copy = match copy_of(address, len, entry, outs, &[STOP, PLACED]) {
        Ok(c) => c,
        Err(why) => {
            t.check(false, || format!("no copy: {why}"));
            return;
        }
    };
    let run = run_copy(&copy);
    let mut object = test_object(0x2468, &[]);
    let mut kind: Vec<u32> = (0..0x22u32).map(|n| 0x3E80_0000 + n * 0x7_1234).collect();
    let (o, k) = (object.as_mut_ptr() as u32, kind.as_mut_ptr() as u32);
    let mut all = vec![(o, OBJECT_LEN), (k, 0x88)];
    all.extend(common_regions());
    let saved = capture(&all);
    let combos: usize = extra.iter().map(|e| e.1.len()).product();
    for id in [0u32, 2, !0, !1] {
        for state in 0..4u32 {
            for outside in [0u32, 1] {
                for combo in 0..combos {
                    let mut fields = vec![(0u32, id), (0x10C, state), (0xC, outside)];
                    let mut rest = combo;
                    for &(at, values) in extra {
                        fields.push((at, values[rest % values.len()]));
                        rest /= values.len();
                    }
                    let fill = || unsafe {
                        let p = o as *mut u32;
                        for &(at, v) in &fields {
                            p.add(at as usize / 4).write(v);
                        }
                        let q = k as *mut u32;
                        for n in [0x10usize, 0x38, 0x60] {
                            q.add(n / 4).write(0x40 + n as u32);
                        }
                    };
                    both(
                        t,
                        &format!("fields {fields:x?}"),
                        &run,
                        Regs {
                            ebx: o,
                            edi: k,
                            esi: game_addr(rva(REQUEST)) as u32,
                            ..Regs::SENTINEL
                        },
                        &fill,
                        &all,
                        rust,
                        false,
                    );
                }
            }
        }
    }
    restore(&all, &saved);
    drop((object, kind));
}
