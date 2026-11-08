//! `race::stop_frame_pictures` (0x41F3C6): the replay's frames no longer saved as pictures
//! (`g_replayRecording` and `dword_5C2CD4` 0; see `race::replay_screenshot`).
//!
//! Caller: `race::viewer_action` (actions 0x25 .. 0x28).

use crate::race::viewer_screen_store::VIEWER_PANEL;

/// The Rust version.
pub fn stop_frame_pictures() {
    VIEWER_PANEL.replay_recording.set(0);
    VIEWER_PANEL.pictures_started.set(0);
}
