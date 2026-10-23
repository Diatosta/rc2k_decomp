//! `hud::overlay_states` (0x45A943): the 2D overlay's render states: 0x15 = 4, 0x1B = 1,
//! 0x13 = 5, 0x14 = 6, 0x1C = 0.
//!
//! In:  nothing
//! Out: eax 0, ebx 0x1C; the rest as the state calls leave them
//!
//! Callers: `sub_45A893` (0x45A8A2), code at 0x4559AF, 0x4559C8. The states through
//! `part` (1 with eax the value, ebx the state), in the game `set_render_state`'s Rust
//! version.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::Regs;

/// The Rust version (see the header).
// Converted: `rc2k_core::scene::render_states::OVERLAY`
pub fn overlay_states() {
    for s in rc2k_core::scene::render_states::OVERLAY {
        crate::scene::set_render_state::set_render_state(s.value, s.state);
    }
}

/// For the typed tables' calls by address. What the original leaves in eax and ebx is
/// read by none of its callers (register liveness, 2026-10-23).
pub fn entry(_r: &mut Regs) -> Result<(), crate::errors::Refused> {
    overlay_states();
    Ok(())
}
