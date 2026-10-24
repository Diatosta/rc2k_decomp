//! `menu::switch_stage_group` (0x481FAB): the stage choice's group switched (0 <-> 1), then the map drawn again.
//!
//! In:  nothing
//! Out: every register preserved (as the map leaves them)
//!
//! See `menu/stage_choice.rs`. In three item lists (the stage choice screens).
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::menu::stage_choice::{draw_stage_map, switch_stage_group};

/// The Rust version the typed tables run.
pub fn run() -> Result<(), Refused> {
    unsafe { switch_stage_group(&mut || draw_stage_map()) }
}
