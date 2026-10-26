//! `physics::stir_car_word` (0x440F2E): `dword_719FE4` (the work copy's +0xC, the word
//! `physics::test_car_word` tests) = `scene::noise_step` of itself.
//!
//! In:  nothing
//! Out: eax the new word; the rest kept (`noise_step` keeps edx); no flags read
//!
//! Caller: `sub_43B635`, its first instruction: every physics step.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::physics::car_store::CAR;
use crate::scene::next_noise::noise_step;

/// The Rust core on the word at `at`: the new word.
pub unsafe fn stir_word_at(at: u32) -> u32 {
    unsafe {
        let p = crate::pointers::host(at) as *mut u32;
        let w = noise_step(p.read_unaligned());
        p.write_unaligned(w);
        w
    }
}

/// The Rust version.
pub unsafe fn stir_car_word() -> u32 {
    unsafe { stir_word_at(CAR.noise_word.addr() as u32) }
}
