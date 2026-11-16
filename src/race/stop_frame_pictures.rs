//! `race::stop_frame_pictures`: stops saving the replay's frames as pictures (`replay_recording`
//! and the picture clock's `started` flag are set to 0; see `race::replay_screenshot`).

use crate::race::owner::Race;

pub fn stop_frame_pictures(race: &Race, clock: &crate::system::tick_count::PictureClock) {
    let panel = race.viewer_panel();
    panel.replay_recording.set(0);
    clock.started.set(0);
}
