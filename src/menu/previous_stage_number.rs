//! `menu::previous_stage_number` (0x48201B): the stage choice's stage back (below 2, signed, to 6), then the map drawn again.
//!
//! In:  nothing
//! Out: every register preserved (as the map leaves them)
//!
//! See `menu/stage_choice.rs`. In three item lists (the stage choice screens). The menus and draw
//! lists run its `entry` (`menu::table_calls`); the registers above are the original's, for
//! reference.

use crate::errors::Refused;
use crate::menu::stage_choice::{draw_stage_map, previous_stage_number};

/// The Rust version the typed tables run.
pub fn run() -> Result<(), Refused> {
    previous_stage_number(&mut || draw_stage_map())
}
