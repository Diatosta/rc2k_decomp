//! `race::start_frame_pictures`: starts saving the replay's frames as pictures
//! (`race::replay_screenshot` saves one each frame). The rate is held to 1 .. 100 (signed); the
//! time a picture takes is the given ms, or for 0 1000 / the rate. The folder `rpl` is made,
//! `replay_recording` is set to 1, and the picture clock (`clock::tick_count::PICTURE_CLOCK`)
//! is reset to 0.
//!
//! Takes the ms a picture takes, or 0 for the rate's.

use crate::errors::Refused;
use crate::race::owner::Race;

/// Starts the picture series; `picture_ms` is the ms a picture takes, or 0 for the rate's.
pub fn start_frame_pictures(
    race: &Race,
    clock: &crate::clock::tick_count::PictureClock,
    picture_ms: u32,
) -> Result<(), Refused> {
    use rc2k_core::race::viewer::{picture_rate, picture_time};
    let panel = race.viewer_panel();
    let rate = picture_rate(panel.picture_rate());
    if panel.picture_rate() != rate {
        panel.set_picture_rate(rate);
    }
    panel.set_picture_time(picture_time(picture_ms, rate));
    // Whether the folder was made does not matter.
    crate::files::directories::make_directory(&crate::files::cstring_in(
        &panel.picture_folder.get().to_le_bytes(),
    )?);
    panel.replay_recording.set(1);
    clock.started.set(0);
    clock.clock.set(0);
    Ok(())
}
