//! `menu::fade_in` (0x4A1CFA): the inspection's fade in: the see-through from 0 up to `dword_75AA94` with the time gone (`menu::fades::fade`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_48E093`, `sub_4A15E3`, `sub_4A8556`. The clock is `system::tick_count`'s Rust version; the frames
//! (`menu::fade_frame` through `screens::run_frame`) through the game (platform).
//! Ungated (it runs for the fade's length). A private copy (a call at +1; its three jumps
//! to the fatal stub relocated). Fatal error 0x80701010 as the original's checks;
//! refuses a length of 1 .. 99 ms (the original divides by 0), before anything. Shadow
//! mode runs the original alone (it shows frames). Self-test: `fades::test_fade`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;

/// The Rust version (see the header).
pub fn fade_in() -> Result<(), Refused> {
    unsafe { crate::menu::fades::fade(false) }
}
