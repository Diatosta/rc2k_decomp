//! `menu::previous_stage_sub` (0x481FD9): the stage choice's rally back (below 1 to 5), then the map drawn again.
//!
//! In:  nothing
//! Out: every register preserved (as the map leaves them)
//!
//! See `menu/stage_choice.rs`. In three item lists (the stage choice screens).
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::menu::stage_choice::{draw_stage_map_in_game, previous_stage_sub};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { previous_stage_sub(&mut || draw_stage_map_in_game()) }?;
    Ok(())
}
