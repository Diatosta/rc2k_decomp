//! `math::stir_random`: stirs a number into the random numbers' state.
//!
//! One step of the generator on the number with the state mixed in; unless that is 0, it
//! becomes the state. Run at start, after `system::read_volume_serial`; the number stirred in
//! is a leftover from the process's start, not the volume's number.

pub fn stir_random(random: &crate::math::next_random::Randoms, number: u32) {
    random.with_random(|r| r.stir(number));
}
