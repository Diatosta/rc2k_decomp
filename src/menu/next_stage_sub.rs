//! `menu::next_stage_sub` (0x481FB8): the stage choice's rally on (5 or more to 0), then the map drawn again.
//!
//! In:  nothing
//! Out: every register preserved (as the map leaves them)
//!
//! See `menu/stage_choice.rs`. In three item lists (the stage choice screens).
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::framework::Regs;
use crate::menu::stage_choice::{draw_stage_map_in_game, next_stage_sub};

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(_: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { next_stage_sub(&mut || draw_stage_map_in_game()) }?;
    Ok(())
}
