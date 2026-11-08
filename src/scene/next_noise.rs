//! `scene::next_noise` (0x440F3E): the next number of the scene's own generator.
//!
//! Returns: the number
//!
//! One step on the state `dword_614C54` (`scene::noise_step`, 0x440F4E, inlined): bit 22, then
//! bits 2 and 1 of the low byte as it then is, each flip bit 0; the whole turned right by one;
//! then `xor` 0x87654321. `math::next_random`'s step with that last `xor` added. Five callers
//! (two in the engine's tables, three unread).

use crate::race::owner_shared::Shared;

// ---- Rust version ----

/// `scene::noise_step` (0x440F4E). Converted: `rc2k_core::scene::noise::step`.
pub fn noise_step(state: u32) -> u32 {
    rc2k_core::scene::noise::step(state)
}

/// Converted: `rc2k_core::scene::noise::Noise::draw`.
pub fn next_noise(shared: &Shared) -> u32 {
    let mut n = rc2k_core::scene::noise::Noise {
        state: shared.physics_tables.noise_state.bits(),
    };
    let number = n.draw();
    shared.physics_tables.noise_state.set_bits(n.state);
    number
}
