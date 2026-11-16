//! `math::stir_random`: stirs a number into the random numbers' state.
//!
//! One step of the generator on the number with the state mixed in; unless that is 0, it
//! becomes the state. Run once at start, with 0.

pub fn stir_random(random: &crate::math::next_random::Randoms, number: u32) {
    random.with_random(|r| r.stir(number));
}
