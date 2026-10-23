//! `drive::duel_sound_levels` (0x45C596): two sound levels from the first car block
//! (`g_carBlockStore`): +0x900 / 4 - 4, held 0 ..= 0x40 (by the signs), into
//! `dword_5165BC`; at 0x38 or more `dword_5165A8` 1 when a random number
//! (`math::next_random`) x 500 has a high dword of 0 (a rare draw); (+0x908 squared, low
//! dword) >> 12 - 8, held, into `dword_5165C0`.
//!
//! In:  nothing
//! Out: eax the second level, edx the square's high dword, ebx 500 after a draw; the rest
//!      kept; no flags read
//!
//! Caller: `drive::replay_step`.
//! Self-test: the original and the Rust version on the figures (saved and put back), the
//! random state included.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{EAX, EBX, EDX, Regs};
use crate::physics::flow::{g, p};
use crate::screens::harness::Left;
use rc2k_core::drive::replay_views::{RARE_FROM, first_level, rare_draw, second_level};

const B: usize = 0x70F3EC; // g_carBlockStore

/// The Rust version.
// Converted: `rc2k_core::drive::replay_views::{first_level, rare_draw, second_level}`.
pub unsafe fn duel_sound_levels(entry: &Regs) -> Left {
    let mut left = Left::nothing();
    p(0x5165BC, first_level(g(B + 0x900)));
    if g(0x5165BC) >= RARE_FROM {
        let r = unsafe { crate::math::next_random::next_random() };
        left = left.with(EBX, 500);
        if rare_draw(r) {
            p(0x5165A8, 1);
        }
    }
    let (eax, high) = second_level(g(B + 0x908));
    p(0x5165C0, eax);
    let _ = entry;
    left.with(EAX, eax).with(EDX, high)
}
