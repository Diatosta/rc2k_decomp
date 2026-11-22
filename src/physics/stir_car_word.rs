//! `noise_word` (the word `physics::car_word_bits` tests) = `scene::noise_step` of itself.
//!
//! Returns the new word.

use crate::physics::car_store::Car;
use crate::scene::next_noise::noise_step;

/// On the word at `at`: the new word.
fn stir_word_at(at: &mut u32) -> u32 {
    let w = noise_step(*at);
    *at = w;
    w
}

pub(super) fn stir_car_word(car: &mut Car) -> u32 {
    stir_word_at(&mut car.noise_word)
}
