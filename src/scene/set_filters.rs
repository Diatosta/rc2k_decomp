//! `scene::set_filters`: the textures' filters, by the setting.
//!
//! With `RaceVars.mip_setting` at 0 render states 0x11 and 0x12 (the filters for a texture made
//! larger and smaller) both 2; else 2 and 6. Then state 0x31 = 1. Each through
//! `scene::set_render_state`.

use crate::race::owner::Race;
use crate::scene::set_render_state::set_render_state;

// ---- game data ----

// Logic: `rc2k_core::scene::render_states::filters`
pub fn set_filters(race: &Race, part: &crate::scene::scratch_store::PartScratch) {
    {
        for s in rc2k_core::scene::render_states::filters(race.shared.vars.get().mip_setting.get())
        {
            set_render_state(part, s.value, s.state);
        }
    }
}
