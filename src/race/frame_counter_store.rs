//! The `framecounter` setting (`FrameCounter.show`): shows the frame rate when not 0
//! (`race::draw_frame_rate`; `settings::read_settings`).

pub struct FrameCounter {
    /// The `framecounter` setting (`race::draw_frame_rate`).
    pub show: u32,
}

impl FrameCounter {
    pub const fn new() -> Self {
        FrameCounter { show: 0 }
    }
}
