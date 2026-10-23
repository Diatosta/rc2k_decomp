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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_read, game_write, rva};
use crate::math::x87;
use crate::physics::fill_race_slot::regions;
use crate::physics::fill_slot_on_bit::fill_slot_on_bit_in;
use crate::physics::test_car_word::test_word;
use crate::screens::harness::Left;
use crate::service::harness::{capture, restore, state_entry};
use rc2k_core::physics::parts::{self, DROP_TESTS, Hit, HitTest};

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

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
