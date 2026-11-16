//! The 2D overlay's render states: 0x15 = 4, 0x1B = 1, 0x13 = 5,
//! 0x14 = 6, 0x1C = 0.

pub fn overlay_states(part: &crate::scene::workers::PartScratch) {
    for s in rc2k_core::scene::render_states::OVERLAY {
        crate::scene::set_render_state::set_render_state(part, s.value, s.state);
    }
}
