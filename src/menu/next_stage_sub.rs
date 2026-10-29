//! `menu::next_stage_sub` (0x481FB8): the stage choice's rally on (5 or more to 0), then the map drawn again.
//!
//! In:  nothing
//! Out: every register preserved (as the map leaves them)
//!
//! See `menu/stage_choice.rs`. In three item lists (the stage choice screens). The menus and draw
//! lists run its `entry` (`menu::table_calls`); the registers above are the original's, for
//! reference.

use crate::errors::Refused;
use crate::menu::stage_choice::{draw_stage_map, next_stage_sub};

/// The Rust version the typed tables run.
pub fn run() -> Result<(), Refused> {
    next_stage_sub(&mut || draw_stage_map())
}
