//! `hud::overlay_states` (0x45A943): the 2D overlay's render states: 0x15 = 4, 0x1B = 1, 0x13 = 5,
//! 0x14 = 6, 0x1C = 0.
//!
//! Callers: `hud::overlay_pass`, code at `race::draw_view_overlays`, `race::draw_view_overlays`. a
//! call by address goes through `routes.rs`.

// Converted: `rc2k_core::scene::render_states::OVERLAY`
pub fn overlay_states(part: &crate::workers::PartScratch) {
    for s in rc2k_core::scene::render_states::OVERLAY {
        crate::scene::set_render_state::set_render_state(part, s.value, s.state);
    }
}
