//! `physics::sound_hit_parts` (0x4411EC): for kinds with a `dword_5C5BD0` entry (below
//! 0x22), the same four tests as `physics::drop_hit_parts` on the timers +0x354 ..
//! +0x360 and the bits 1 .. 4 of +0x298, by `physics::sound_on_bit` (the timer 256).
//!
//! In:  nothing
//! Out: as the calls leave them; CF when the kind has no entry (no caller reads it)
//!
//! Caller: `physics::hit_part_checks`. Refuses what `sound_on_bit` refuses.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::physics::drop_hit_parts::{Timers, g, hit_tests};
use crate::physics::sound_on_bit::sound_on_bit;
use rc2k_core::physics::parts::{SOUND_TESTS, has_hit_sounds};

const TIMERS: Timers = [0x71A32C, 0x71A330, 0x71A334, 0x71A338];

/// The Rust version (its CF, set when the car has no hit sounds, is read by no caller).
// Converted: `rc2k_core::physics::parts::{has_hit_sounds, hit_tests, SOUND_TESTS}`.
pub fn sound_hit_parts() -> Result<(), Refused> {
    if !has_hit_sounds(g(0x719FFC), |kind| g(0x5C5BD0 + 4 * kind as usize)) {
        return Ok(());
    }
    hit_tests(&SOUND_TESTS, &TIMERS, &mut |bit| {
        unsafe { sound_on_bit(bit & 0xFF) }?;
        Ok(())
    })
}
