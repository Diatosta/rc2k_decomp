//! `menu::fade_in` (0x4A1CFA): the inspection's fade in: the see-through from 0 up to `dword_75AA94` with the time gone (`menu::fades::fade`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_48E093`, `sub_4A15E3`, `sub_4A8556`. The clock is `system::tick_count`'s Rust
//! version; the frames (`menu::fade_frame` through `screens::run_frame`) through the game
//! (platform). Fatal error 0x80701010 as the original's checks; refuses a length of 1 .. 99 ms (the
//! original divides by 0), before anything.

use crate::errors::Refused;

/// The Rust version (see the header).
pub fn fade_in() -> Result<(), Refused> {
    unsafe { crate::menu::fades::fade(false) }
}
