//! `menu::update_frame_clock` (0x411019): stores `tick_count` in `dword_51899C`, the clock the
//! menu code measures blinking and idle time against.
//!
//! In:  nothing
//! Out: eax = the clock; everything else preserved
//! Three callers.

use crate::session::CLOCKS;
use crate::system::tick_count::tick_count;

// ---- game data ----

// ---- Rust version ----

pub fn update_frame_clock() -> u32 {
    let now = tick_count();
    CLOCKS.frame.set(now);
    now
}
