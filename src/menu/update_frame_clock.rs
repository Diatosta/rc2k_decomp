//! `menu::update_frame_clock` (0x411019): stores `tick_count` in the window handle, the clock the
//! menu code measures blinking and idle time against.
//!
//! Three callers.

use crate::system::owner::system;
use crate::system::tick_count::tick_count;

// ---- game data ----

pub fn update_frame_clock() -> u32 {
    let now = tick_count();
    system().session.clocks.frame.set(now);
    now
}
