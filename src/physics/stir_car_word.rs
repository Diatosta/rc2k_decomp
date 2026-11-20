//! `noise_word` (the word `physics::car_word_bits` tests) = `scene::noise_step` of itself.
//!
//! Returns the new word.

use crate::memory::session::PlainDword;
use crate::physics::car_store::Car;
use crate::scene::next_noise::noise_step;

/// On the word at `at`: the new word.
pub fn stir_word_at(at: &PlainDword) -> u32 {
    let w = noise_step(at.get());
    at.set(w);
    w
}

pub fn stir_car_word(car: &Car) -> u32 {
    stir_word_at(&car.noise_word)
}
