//! `race::replay_mode_2` (0x41CD9D): the replay viewer with eax 2 (a saved replay chosen
//! first), with no check of a replay there (unlike `replay_mode_1` / `_3`).
//!
//! In:  nothing
//! Out: eax 2, the rest as the viewer leaves them (kept)
//!
//! Caller: `sub_470529` (a menu item).

use crate::errors::Refused;

/// The Rust version.
pub fn replay_mode_2() -> Result<(), Refused> {
    crate::race::replay_viewer::replay_viewer(crate::race::replay_viewer::Mode::Saved)
}
