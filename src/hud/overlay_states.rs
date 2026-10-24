//! `hud::overlay_states` (0x45A943): the 2D overlay's render states: 0x15 = 4, 0x1B = 1,
//! 0x13 = 5, 0x14 = 6, 0x1C = 0.
//!
//! In:  nothing
//! Out: eax 0, ebx 0x1C; the rest as the state calls leave them
//!
//! Callers: `sub_45A893` (0x45A8A2), code at 0x4559AF, 0x4559C8. The registers above are the
//! original's, for reference; a call by address goes through `routes.rs`.

/// The Rust version (see the header).
// Converted: `rc2k_core::scene::render_states::OVERLAY`
pub fn overlay_states() {
    for s in rc2k_core::scene::render_states::OVERLAY {
        crate::scene::set_render_state::set_render_state(s.value, s.state);
    }
}
