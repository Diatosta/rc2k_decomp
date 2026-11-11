//! `system::tick_count` (0x411024): the game's millisecond clock: `GetTickCount`, unless the
//! override switch (`pictures_started`) is 1, in which case it is `picture_clock`.
//!
//! Returns milliseconds.
//! 110 callers. What sets the override has not been read.
//!

use crate::race::viewer_screen_store::VIEWER_PANEL;

// ---- game data ----

pub fn tick_count() -> u32 {
    #[cfg(feature = "remote")]
    if let Some(held) = crate::remote::held_clock() {
        return held;
    }
    // The original asks Windows first in both cases.
    let now = crate::os::ticks();
    // Converted: `rc2k_core::system::time::clock`.
    rc2k_core::system::time::clock(now, VIEWER_PANEL.pictures_started.bits(), || {
        VIEWER_PANEL.picture_clock.bits()
    })
}
