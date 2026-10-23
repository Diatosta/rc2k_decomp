//! `menu::fade_out` (0x4A1DA5): the inspection's fade out: the see-through from `dword_75AA94` down (1 while the share is 0) (`menu::fades::fade`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: `sub_4A8556`. The clock is `system::tick_count`'s Rust version; the frames
//! (`menu::fade_frame` through `screens::run_frame`) through the game (platform).
//! Ungated (it runs for the fade's length). A private copy (a call at +1; its three jumps
//! to the fatal stub relocated). Fatal error 0x80701010 as the original's checks (here also a time gone of 0 at the first frame: section 16);
//! refuses a length of 1 .. 99 ms (the original divides by 0), before anything. Shadow
//! mode runs the original alone (it shows frames). Self-test: `fades::test_fade`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::menu::fades::{fade, game_part};

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { fade(r, true, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
