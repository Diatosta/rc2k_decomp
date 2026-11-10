//! `physics::stir_car_word` (0x440F2E): `dword_719FE4` (the work copy's +0xC, the word
//! `physics::test_car_word` tests) = `scene::noise_step` of itself.
//!
//! Returns: the new word
//!
//! Caller: `physics::car_physics_step`, its first instruction: every physics step.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::race::owner_shared::Shared;
use crate::scene::next_noise::noise_step;

/// On the word at `at`: the new word.
pub fn stir_word_at(at: &crate::session::Dword) -> u32 {
    let w = noise_step(at.get());
    at.set(w);
    w
}

pub fn stir_car_word(shared: &Shared) -> u32 {
    stir_word_at(&shared.car.noise_word)
}
