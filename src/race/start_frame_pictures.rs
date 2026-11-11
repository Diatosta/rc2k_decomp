//! `race::start_frame_pictures` (0x41F35E): the replay's frames to be saved as pictures
//! (`race::replay_screenshot` does it each frame): the rate held to 1 .. 100 (signed); the time a
//! picture takes is the given ms, or for 0 1000 / the rate; the folder `rpl` made;
//! `g_replayRecording` 1, `pictures_started` and `picture_clock` 0.
//!
//! Takes: the ms a picture, or 0 for the rate's
//!
//! Caller: `race::viewer_action` (actions 0x25 .. 0x28: 0, 0xA0, 0x50, 0x28).

use crate::errors::Refused;
use crate::race::owner::Race;

/// `ms` the time a picture takes, or 0 for the rate's.
pub fn start_frame_pictures(race: &Race, eax: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::viewer::{picture_rate, picture_time}`.
    use rc2k_core::race::viewer::{picture_rate, picture_time};
    let panel = race.viewer_panel();
    let rate = picture_rate(panel.picture_rate());
    if panel.picture_rate() != rate {
        panel.set_picture_rate(rate);
    }
    panel.set_picture_time(picture_time(eax, rate));
    // The folder made (whether it was is not read).
    crate::files::directories::make_directory(&crate::files::cstring(
        &panel.picture_folder,
        "the pictures folder",
    )?);
    panel.replay_recording.set(1);
    panel.pictures_started.set(0);
    panel.picture_clock.set(0);
    Ok(())
}
