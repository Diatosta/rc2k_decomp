//! `menu::arcade_level_5` (0x487DE5): the arcade start menu's level 5 item (see
//! `menu/arcade_levels.rs`).
//!
//! In:  nothing
//! Out: eax -1 for a closed level; else as the event leaves them
//! Hook removed 2026-10-22 (typed tables): the menus and draw lists run its `entry`
//! (`menu::table_calls`); the registers above are the original's, for reference.

use crate::errors::Refused;
use crate::menu::arcade_levels::arcade_level;
use crate::menu::screen_loop::Carried;

/// The Rust version (`c` the carried values the item gets).
pub unsafe fn arcade_level_5(c: &Carried) -> Result<(), Refused> {
    unsafe { arcade_level(c, 5) }
}
