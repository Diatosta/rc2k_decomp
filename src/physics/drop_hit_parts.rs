//! `physics::drop_hit_parts` (0x4410C8): for cars of kind (+0x24) below 0x17, four parts
//! in turn (`physics::test_car_word` bits 5, 9, 0xE, 0x19 of +0xC; their timers +0x2F8 ..
//! +0x304): the first whose bit is set and timer not below 100 (`flt_5C5D14`) loses its
//! part when its bit of +0x298 (5, 6, 7, 8) is still set (`physics::fill_slot_on_bit`, the
//! timer 256) and it ends there; a bit of +0x298 already clear ends it too. A reading:
//! hit parts coming off.
//!
//! In:  nothing
//! Out: as the calls leave them; no flags read (the caller calls on)
//!
//! Caller: `physics::hit_part_checks`.
//! Refuses what `fill_slot_on_bit` refuses; what was written is put back.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back): words, timers and bits by lot.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::math::x87;
use crate::physics::fill_race_slot::{inputs, regions, set_slots};
use crate::physics::fill_slot_on_bit::fill_slot_on_bit_in;
use crate::physics::test_car_word::test_word;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore, state_entry};
use rc2k_core::physics::parts::{self, DROP_TESTS, Hit, HitTest};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("physics::drop_hit_parts", rva(0x4410C8), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn g(x: usize) -> u32 {
    unsafe { game_read(rva(x)) }
}

pub fn p(x: usize, v: u32) {
    unsafe { game_write(rva(x), v) }
}

/// The four tests' timers (IDA addresses).
pub type Timers = [usize; 4];

/// The shared steps (`rc2k_core::physics::parts::hit_tests`): `act` the bit's step (with
/// ecx the bit).
pub fn hit_tests(
    r: &mut Regs,
    tests: &[HitTest; 4],
    timers: &Timers,
    act: &mut dyn FnMut(&mut Regs) -> Result<(), Refused>,
) -> Result<(), Refused> {
    let word = g(0x719FE4);
    let (last, hit) = parts::hit_tests(
        tests,
        word,
        |k| f32::from_bits(g(timers[k])),
        || f32::from_bits(g(0x5C5D14)),
        || g(0x71A270),
    );
    r.ecx = tests[last].0 as u32;
    test_word(word, tests[last].0).apply(r);
    if let Hit::Act(k) = hit {
        r.ecx = tests[k].1;
        act(r)?;
        p(timers[k], x87::loaded(g(0x5C5E2C)));
    }
    Ok(())
}

const TIMERS: Timers = [0x71A2D0, 0x71A2D4, 0x71A2D8, 0x71A2DC];

/// The steps on `r`.
// Converted: `rc2k_core::physics::parts::{hit_tests, DROP_TESTS}`.
pub fn drop_hit_parts_in(r: &mut Regs) -> Result<(), Refused> {
    if g(0x719FFC) >= 0x17 {
        return Ok(());
    }
    hit_tests(r, &DROP_TESTS, &TIMERS, &mut |r| fill_slot_on_bit_in(r))
}

/// The Rust version.
pub unsafe fn drop_hit_parts(entry: &Regs) -> Result<Left, Refused> {
    let all = regions();
    let saved = capture(&all);
    let mut r = *entry;
    match drop_hit_parts_in(&mut r) {
        Ok(()) => Ok(crate::race::stage_prefs::left_of(&r)),
        Err(e) => {
            restore(&all, &saved);
            Err(e)
        }
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { drop_hit_parts(r) }
}

fn all_regions() -> Vec<Region> {
    let mut r = regions();
    r.push((game_addr(rva(0x71A2D0)) as u32, 0x10));
    r
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &all_regions(), &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

/// The hit tests' inputs: the kind, the word, the timers, the bits.
pub fn set_hits(seed: u32) {
    p(0x719FFC, [3u32, 0x16, 0x17, 0x21, 0][(seed % 5) as usize]);
    p(0x719FE4, seed.wrapping_mul(0x9E37_79B9));
    for k in 0..4usize {
        let v = [50.0f32, 100.0, 300.0, f32::NAN][(seed as usize + k) % 4];
        p(0x71A2D0 + 4 * k, v.to_bits());
        p(0x71A32C + 4 * k, v.to_bits());
    }
    p(0x71A270, 0xFFFF_FFFF ^ (seed.wrapping_mul(0x24) & 0x1FE));
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut all = all_regions();
    all.extend(inputs());
    all.extend([
        (game_addr(rva(0x719FE4)) as u32, 4),
        (game_addr(rva(0x719FFC)) as u32, 4),
        // `set_hits` writes these too (left changed until 2026-10-18).
        (game_addr(rva(0x71A2D0)) as u32, 0x10),
        (game_addr(rva(0x71A32C)) as u32, 0x10),
        (game_addr(rva(0x71A270)) as u32, 4),
    ]);
    let saved = capture(&all);
    for seed in 0..20u32 {
        check_state(
            &mut t,
            &format!("seed {seed}"),
            call_original,
            Regs {
                ebp: 0x1234,
                ..Regs::SENTINEL
            },
            &|| {
                set_slots(seed, Some(seed % 4));
                p(0x71B3F8, seed % 8);
                set_hits(seed);
            },
            &all,
            &rust,
        );
    }
    restore(&all, &saved);
    t.finish()
}
