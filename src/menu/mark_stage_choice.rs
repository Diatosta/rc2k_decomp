//! `menu::mark_stage_choice` (0x486C21): `dword_755700` 1 (the stage screens' map then takes
//! the choice's stage; 2 a championship's, 3 a record's).
//!
//! In:  nothing
//! Out: every register preserved
//!
//! One caller, `menu::select_stage_loop` at its set-up. Rust callers run its `entry` (`routes.rs`);
//! the registers above are the original's, for reference.

use crate::menu::owner::menus;

pub fn mark_stage_choice() {
    // Converted: `rc2k_core::menu::event::MapStage`.
    menus()
        .play
        .champ_view
        .stage_mode
        .set(rc2k_core::menu::event::MapStage::Choice as u32);
}
