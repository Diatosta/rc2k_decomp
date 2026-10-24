//! `menu::update_frame_clock` (0x411019): stores `tick_count` in `dword_51899C`, the clock the
//! menu code measures blinking and idle time against.
//!
//! In:  nothing
//! Out: eax = the clock; everything else preserved
//! Three callers. The clock moves, so shadow mode and the self-test accept any value between
//! the Rust answers from before and after the original.
//! Manual patch with a private copy (0x0B bytes, the call relocated): the function starts
//! with a call.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::session::CLOCKS;
use crate::system::tick_count::tick_count;

// ---- game data ----

// ---- Rust version ----

pub fn update_frame_clock() -> u32 {
    let now = tick_count();
    CLOCKS.frame.set(now);
    now
}
