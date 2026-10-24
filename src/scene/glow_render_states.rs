//! `scene::glow_render_states` (0x44E44D): three of Direct3D's render states, for what is
//! drawn glowing (the sky's sun, the views' backdrop): 7 = 1 (the depth test on), 0xE = 1
//! (depth writing on), 0x17 = 8 (the comparison: always), each by `scene::set_render_state`.
//!
//! In:  nothing
//! Out: eax = 8, ebx = 0x17 (the last state's); the rest kept; no flags read
//!
//! Callers: `race::draw_sky`, `race::draw_view_backdrop`.

/// The Rust version (see the header): the registers it leaves on `r`.
// Converted: `rc2k_core::scene::render_states::GLOW`
pub fn glow_render_states() {
    for s in rc2k_core::scene::render_states::GLOW {
        crate::scene::set_render_state::set_render_state(s.value, s.state);
    }
}
