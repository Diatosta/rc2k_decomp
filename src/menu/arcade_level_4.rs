//! `menu::arcade_level_4` (0x487D9E): the arcade start menu's level 4 item (see
//! `menu/arcade_levels.rs`).
//!
//! In:  nothing
//! Out: eax -1 for a closed level; else as the event leaves them
//! The menus and draw lists run its `entry` (`menu::table_calls`); the registers above are the
//! original's, for reference.

use crate::errors::Refused;
use crate::menu::arcade_levels::arcade_level;

/// The Rust version.
pub fn arcade_level_4() -> Result<(), Refused> {
    arcade_level(4)
}
