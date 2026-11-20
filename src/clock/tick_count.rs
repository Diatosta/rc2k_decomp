//! The game's millisecond clock: the machine's, unless a replay is rendered to a video (the
//! video's own clock), a remote run holds the clock, or the replay viewer is saving its frames
//! as pictures (the `PictureClock` override).

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

/// Where a replay rendered to a video gets its time from; none while no video is rendered.
static VIDEO_CLOCK: std::sync::OnceLock<fn() -> Option<u32>> = std::sync::OnceLock::new();

/// Names the video's clock: while it gives a time, that is the game's time. Set once, by the
/// module that renders the video.
pub fn use_video_clock(source: fn() -> Option<u32>) {
    // A second call names the same function: the first stays.
    let _ = VIDEO_CLOCK.set(source);
}

pub fn tick_count(picture_clock: &PictureClock) -> u32 {
    // A replay rendered to a video runs on the video's own clock.
    if let Some(video) = VIDEO_CLOCK.get().and_then(|source| source()) {
        return video;
    }
    if let Some(held) = crate::remote::held_clock() {
        return held;
    }
    // The original asks Windows first in both cases.
    let now = crate::platform::os::ticks();
    rc2k_core::system::time::clock(now, picture_clock.started.get(), || {
        picture_clock.clock.get()
    })
}

/// Whether `tick_count` is the machine's own clock right now, so that it moves with real
/// time (not a video's clock, a held race's or the picture clock's override).
pub fn follows_machine_clock(picture_clock: &PictureClock) -> bool {
    VIDEO_CLOCK.get().and_then(|source| source()).is_none()
        && crate::remote::held_clock().is_none()
        && picture_clock.started.get() != 1
}
