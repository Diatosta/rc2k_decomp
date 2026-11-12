//! `scene::set_filters` (0x44E8FB): the textures' filters, by the setting.
//!
//! With `RaceVars.mip_setting` at 0 render states 0x11 and 0x12 (the filters for a texture made larger
//! and smaller) both 2; else 2 and 6. Then state 0x31 = 1. Each through
//! `scene::set_render_state`. Five callers, `scene::begin_scene` among them. The registers
//! above are the original's; a call by address goes through `routes.rs`, which maps them.

use crate::race::owner::Race;
use crate::scene::set_render_state::set_render_state;

// ---- game data ----

// ---- Rust version ----

// Converted: `rc2k_core::scene::render_states::filters`
pub fn set_filters(race: &Race, part: &crate::workers::PartScratch) {
    {
        for s in rc2k_core::scene::render_states::filters(race.shared.vars.get().mip_setting.get())
        {
            set_render_state(part, s.value, s.state);
        }
    }
}
