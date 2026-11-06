//! `scene::set_fog_colour` (0x44E76B): the fog's colour: eax kept in `dword_5AB964` and
//! given to Direct3D (`d3d_set_fog_colour`, 0x41AE25: inside `pusha`, render state 0x22 =
//! eax with its top byte set, through `d3d::set_render_state`).
//!
//! In:  eax = the colour
//! Out: every register kept; no flags read
//!
//! Callers: `race::set_up_fog` and ten more. The layer's call is made here (the two lines of
//! `d3d_set_fog_colour`).

/// The Rust version (see the header).
// Converted: `rc2k_core::scene::render_states::fog_colour`
pub fn set_fog_colour(colour: u32) {
    {
        crate::d3d::owner::d3d().cards.fade_from.set(colour);
        let s = rc2k_core::scene::render_states::fog_colour(colour);
        crate::d3d::set_render_state::set_render_state(s.value, s.state);
    }
}
