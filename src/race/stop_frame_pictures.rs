//! `race::stop_frame_pictures`: stops saving the replay's frames as pictures (`replay_recording`
//! and the picture clock's `started` flag are set to 0; see `race::replay_screenshot`).
//!
//! Called by `race::viewer_action` (actions 0x25 .. 0x28).

use crate::race::owner::Race;

pub fn stop_frame_pictures(race: &Race) {
    let panel = race.viewer_panel();
    panel.replay_recording.set(0);
    crate::system::tick_count::PICTURE_CLOCK.started.set(0);
}
