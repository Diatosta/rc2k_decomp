//! `scene::set_filters` (0x44E8FB): the textures' filters, by the setting.
//!
//! With `dword_623898` at 0 render states 0x11 and 0x12 (the filters for a texture made larger
//! and smaller) both 2; else 2 and 6. Then state 0x31 = 1. Each through
//! `scene::set_render_state`. Five callers, `scene::begin_scene` among them. The registers
//! above are the original's; a call by address goes through `routes.rs`, which maps them.

use crate::race::owner::race;
use crate::scene::set_render_state::set_render_state;

// ---- game data ----

// ---- Rust version ----

// Converted: `rc2k_core::scene::render_states::filters`
pub fn set_filters() {
    {
        for s in rc2k_core::scene::render_states::filters(race().shared.vars.mip_setting.bits()) {
            set_render_state(s.value, s.state);
        }
    }
}
