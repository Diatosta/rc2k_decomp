//! `race::stop_frame_pictures` (0x41F3C6): the replay's frames no longer saved as pictures
//! (`g_replayRecording` and `dword_5C2CD4` 0; see `race::replay_screenshot`).
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `race::viewer_action` (actions 0x25 .. 0x28). Shadow mode compares. Self-test:
//! the two filled, against the original.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::race::viewer::{PICTURES_STARTED, RECORDING, put};

/// The Rust version.
pub fn stop_frame_pictures() {
    put(RECORDING, 0);
    put(PICTURES_STARTED, 0);
}
