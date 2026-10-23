//! `physics::sound_hit_parts` (0x4411EC): for kinds with a `dword_5C5BD0` entry (below
//! 0x22), the same four tests as `physics::drop_hit_parts` on the timers +0x354 ..
//! +0x360 and the bits 1 .. 4 of +0x298, by `physics::sound_on_bit` (the timer 256).
//!
//! In:  nothing
//! Out: as the calls leave them; CF when the kind has no entry (no caller reads it)
//!
//! Caller: `physics::hit_part_checks`.
//! Refuses what `sound_on_bit` refuses.
//! Self-test: the original and the Rust version on the game's variables (saved and put
//! back) with a test sound object.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::drop_hit_parts::{Timers, g, hit_tests};
use crate::physics::sound_on_bit::sound_on_bit;
use rc2k_core::physics::parts::{SOUND_TESTS, has_hit_sounds};

const TIMERS: Timers = [0x71A32C, 0x71A330, 0x71A334, 0x71A338];

/// The steps on `r`.
// Converted: `rc2k_core::physics::parts::{has_hit_sounds, hit_tests, SOUND_TESTS}`.
pub fn sound_hit_parts_in(r: &mut Regs) -> Result<(), Refused> {
    if !has_hit_sounds(g(0x719FFC), |kind| g(0x5C5BD0 + 4 * kind as usize)) {
        r.set_carry(true);
        return Ok(());
    }
    hit_tests(r, &SOUND_TESTS, &TIMERS, &mut |r| {
        unsafe { sound_on_bit(r.ecx & 0xFF) }?.apply(r);
        Ok(())
    })
}
