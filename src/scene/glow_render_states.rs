//! `scene::glow_render_states`: three of Direct3D's render states, for what is drawn glowing (the
//! sky's sun, the views' backdrop): 7 = 1 (the depth test on), 0xE = 1 (depth writing on), 0x17 = 8
//! (the comparison: always), each by `scene::set_render_state`.

/// The three states set.
// Logic: `rc2k_core::scene::render_states::GLOW`
pub fn glow_render_states(part: &crate::scene::scratch_store::PartScratch) {
    for s in rc2k_core::scene::render_states::GLOW {
        crate::scene::set_render_state::set_render_state(part, s.value, s.state);
    }
}
