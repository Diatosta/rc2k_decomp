//! `physics::stir_car_word` (0x440F2E): `dword_719FE4` (the work copy's +0xC, the word
//! `physics::test_car_word` tests) = `scene::noise_step` of itself.
//!
//! In:  nothing
//! Out: eax the new word; the rest kept (`noise_step` keeps edx); no flags read
//!
//! Caller: `sub_43B635`, its first instruction: every physics step.
//! Self-test: a copy of the original on a test word (its call to `noise_step` kept),
//! 64 words.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, Regs, game_addr, rva};
use crate::scene::next_noise::noise_step;
use crate::screens::harness::Left;
use crate::service::harness::state_entry;

const WORD_RVA: usize = rva(0x719FE4); // dword_719FE4

/// The Rust core on the word at `at`: the new word.
pub unsafe fn stir_word_at(at: u32) -> u32 {
    unsafe {
        let p = at as *mut u32;
        let w = noise_step(p.read_unaligned());
        p.write_unaligned(w);
        w
    }
}

/// The Rust version.
pub unsafe fn stir_car_word() -> u32 {
    unsafe { stir_word_at(game_addr(WORD_RVA) as u32) }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    Ok(Left::nothing().with(EAX, unsafe { stir_car_word() }))
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}
