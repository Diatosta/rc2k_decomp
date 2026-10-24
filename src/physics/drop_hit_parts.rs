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
//! Caller: `physics::hit_part_checks`. Refuses what `fill_slot_on_bit` refuses; what was
//! written is put back.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::{game_read, game_write, rva};
use crate::math::x87;
use crate::physics::fill_slot_on_bit::fill_slot_on_bit;
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
/// the bit, the original's ecx). The registers left are read by none of the callers.
pub fn hit_tests(
    tests: &[HitTest; 4],
    timers: &Timers,
    act: &mut dyn FnMut(u32) -> Result<(), Refused>,
) -> Result<(), Refused> {
    let word = g(0x719FE4);
    let (_, hit) = parts::hit_tests(
        tests,
        word,
        |k| f32::from_bits(g(timers[k])),
        || f32::from_bits(g(0x5C5D14)),
        || g(0x71A270),
    );
    if let Hit::Act(k) = hit {
        act(tests[k].1)?;
        p(timers[k], x87::loaded(g(0x5C5E2C)));
    }
    Ok(())
}

const TIMERS: Timers = [0x71A2D0, 0x71A2D4, 0x71A2D8, 0x71A2DC];

/// The Rust version.
// Converted: `rc2k_core::physics::parts::{hit_tests, DROP_TESTS}`.
pub fn drop_hit_parts() -> Result<(), Refused> {
    if g(0x719FFC) >= 0x17 {
        return Ok(());
    }
    hit_tests(&DROP_TESTS, &TIMERS, &mut fill_slot_on_bit)
}
