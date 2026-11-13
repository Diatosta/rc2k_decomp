//! The game's millisecond clock: `GetTickCount`, unless the
//! override switch (`pictures_started`) is 1, in which case it is `picture_clock`.
//!
//! What sets the override has not been read.
//!

use crate::session::Dword;

/// The picture clock: while the replay viewer saves its frames as pictures
/// (`race::replay_screenshot`) the game's clock is `clock` (ms) instead of the host's.
/// They were `ViewerAndKinds.viewer_panel.pictures_started` / `picture_clock` (the panel's
/// bytes at +0x3b8 / +0x3b4 stay, unused).
pub struct PictureClock {
    /// 1 when the override is on.
    pub started: Dword,
    pub clock: Dword,
}

pub static PICTURE_CLOCK: PictureClock = PictureClock {
    started: Dword::new(0),
    clock: Dword::new(0),
};

pub fn tick_count() -> u32 {
    #[cfg(feature = "remote")]
    if let Some(held) = crate::remote::held_clock() {
        return held;
    }
    // The original asks Windows first in both cases.
    let now = crate::os::ticks();
    rc2k_core::system::time::clock(now, PICTURE_CLOCK.started.get(), || {
        PICTURE_CLOCK.clock.get()
    })
}
