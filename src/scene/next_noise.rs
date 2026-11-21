//! `scene::next_noise`: the next number of the scene's own generator.
//!
//! One step on the state `PhysicsRun.noise_state` (`scene::noise_step`, inlined): bit 22, then
//! bits 2 and 1 of the low byte as it then is, each flip bit 0; the whole turned right by one; then
//! `xor` 0x87654321. `math::next_random`'s step with that last `xor` added.

use crate::race::owner_shared::RaceShared;

/// `scene::noise_step`. Logic: `rc2k_core::scene::noise::step`.
pub fn noise_step(state: u32) -> u32 {
    rc2k_core::scene::noise::step(state)
}

/// Logic: `rc2k_core::scene::noise::Noise::draw`.
pub fn next_noise(shared: &RaceShared) -> u32 {
    let mut n = rc2k_core::scene::noise::Noise {
        state: shared.run().noise_state,
    };
    let number = n.draw();
    shared.run_mut().noise_state = n.state;
    number
}
