//! `race::stop_frame_pictures` (0x41F3C6): the replay's frames no longer saved as pictures
//! (`g_replayRecording` and `pictures_started` 0; see `race::replay_screenshot`).
//!
//! Caller: `race::viewer_action` (actions 0x25 .. 0x28).

use crate::race::owner::Race;

pub fn stop_frame_pictures(race: &Race) {
    let panel = race.viewer_panel();
    panel.replay_recording.set(0);
    crate::system::tick_count::PICTURE_CLOCK.started.set(0);
}
