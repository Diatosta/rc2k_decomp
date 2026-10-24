//! `scene::next_noise` (0x440F3E): the next number of the scene's own generator.
//!
//! In:  nothing
//! Out: eax = the number; everything else preserved
//!
//! One step on the state `dword_614C54` (`scene::noise_step`, 0x440F4E, inlined): bit
//! 22, then bits 2 and 1 of the low byte as it then is, each flip bit 0; the whole
//! turned right by one; then `xor` 0x87654321. `math::next_random`'s step with that
//! last `xor` added.
//! Five callers (two in the engine's tables, three unread).
//! Self-test: 200 states.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::bridge::Global;
use crate::framework::rva;

// ---- game data ----

pub const STATE_RVA: usize = rva(0x614C54); // dword_614C54
const STATE: Global<u32> = Global::at(0x614C54);

// ---- Rust version ----

/// `scene::noise_step` (0x440F4E). Converted: `rc2k_core::scene::noise::step`.
pub fn noise_step(state: u32) -> u32 {
    rc2k_core::scene::noise::step(state)
}

/// Converted: `rc2k_core::scene::noise::Noise::draw`.
pub unsafe fn next_noise() -> u32 {
    let mut n = rc2k_core::scene::noise::Noise { state: STATE.get() };
    let number = n.draw();
    STATE.set(n.state);
    number
}
