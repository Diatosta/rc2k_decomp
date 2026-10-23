//! `scene::set_filters` (0x44E8FB): the textures' filters, by the setting.
//!
//! In:  nothing
//! Out: eax = 1, ebx = 0x31; everything else preserved
//!
//! With `dword_623898` at 0 render states 0x11 and 0x12 (the filters for a texture
//! made larger and smaller) both 2; else 2 and 6. Then state 0x31 = 1. Each through
//! `scene::set_render_state`. Five callers, `scene::begin_scene` among them.
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::{Regs, game_read, rva};
use crate::scene::set_render_state::set_render_state;

// ---- game data ----

const SETTING_RVA: usize = rva(0x623898); // dword_623898

// ---- Rust version ----

// Converted: `rc2k_core::scene::render_states::filters`
pub fn set_filters() {
    unsafe {
        for s in rc2k_core::scene::render_states::filters(game_read(SETTING_RVA)) {
            set_render_state(s.value, s.state);
        }
    }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(_r: &mut Regs) -> Result<(), crate::errors::Refused> {
    set_filters();
    Ok(())
}
