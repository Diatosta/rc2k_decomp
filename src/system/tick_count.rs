//! The game's millisecond clock: `GetTickCount`, unless the
//! override switch (`pictures_started`) is 1, in which case it is `picture_clock`.
//!
//! What sets the override has not been read.
//!

use crate::memory::session::Dword;

/// The picture clock: while the replay viewer saves its frames as pictures
/// (`race::replay_screenshot`) the game's clock is `clock` (ms) instead of the host's.
#[derive(rc2k_derive::Cells)]
pub struct PictureClock {
    /// 1 when the override is on.
    pub started: Dword,
    pub clock: Dword,
}

impl PictureClock {
    pub const fn new() -> Self {
        PictureClock {
            started: Dword::new(0),
            clock: Dword::new(0),
        }
    }
}

pub fn tick_count(picture_clock: &PictureClock) -> u32 {
    // A replay rendered to a video runs on the video's own clock.
    if let Some(video) = crate::video::clock() {
        return video;
    }
    #[cfg(feature = "remote")]
    if let Some(held) = crate::remote::held_clock() {
        return held;
    }
    // The original asks Windows first in both cases.
    let now = crate::platform::os::ticks().wrapping_add(crate::video::clock_offset());
    rc2k_core::system::time::clock(now, picture_clock.started.get(), || {
        picture_clock.clock.get()
    })
}
