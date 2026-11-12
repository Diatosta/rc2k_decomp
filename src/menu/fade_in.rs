//! `menu::fade_in` (0x4A1CFA): the inspection's fade in: the see-through from 0 up to the fade's full see-through with the time gone (`menu::fades::fade`).
//!
//! The clock is `system::tick_count`'s Rust
//! version; the frames (`menu::fade_frame` through `screens::run_frame`) through the game
//! (platform). Fatal error 0x80701010 as the original's checks; refuses a length of 1 .. 99 ms (the
//! original divides by 0), before anything.

use crate::errors::Refused;
use crate::menu::owner::Menus;

pub fn fade_in(menus: &Menus) -> Result<(), Refused> {
    crate::menu::fades::fade(menus, false)
}
