//! `menu::fade_in`: the inspection's fade in: the see-through from 0 up to the fade's full
//! see-through with the time gone (`menu::fades::fade`).
//!
//! The clock is `clock::tick_count`; the frames (`menu::fade_frame` through
//! `screens::run_frame`) go through the game (platform). Fatal error 0x80701010 for a length of
//! 0; refuses a length of 1 .. 99 ms (it would divide by 0), before anything.

use crate::errors::Refused;
use crate::owners::Owners;

pub fn fade_in(o: Owners<'static>) -> Result<(), Refused> {
    crate::menu::fades::fade(o, false)
}
