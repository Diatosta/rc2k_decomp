//! `physics::stir_car_word` (0x440F2E): `noise_word` (the work copy's +0xC, the word
//! `physics::test_car_word` tests) = `scene::noise_step` of itself.
//!
//! Returns: the new word
//!
//! Caller: `physics::car_physics_step`, its first instruction: every physics step.

use crate::physics::car_store::Car;
use crate::scene::next_noise::noise_step;
use crate::session::PlainDword;

/// On the word at `at`: the new word.
pub fn stir_word_at(at: &PlainDword) -> u32 {
    let w = noise_step(at.get());
    at.set(w);
    w
}

pub fn stir_car_word(car: &Car) -> u32 {
    stir_word_at(&car.noise_word)
}
