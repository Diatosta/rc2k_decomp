//! `race::close_viewed_replay` (0x41EB7C): the viewer's replay closed, if open
//! (`dword_5BEC88` 1): the game's physics patches back (`race::restore_replay_patches`),
//! the file `h` closed (`files::close`), `dword_5BEC88` 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Callers: `race::replay_viewer`, `race::viewer_action` (action 0x14),
//! `race::open_viewed_replay` (failing), `race::save_last_replay`, 0x444034.

use crate::errors::Refused;
use crate::race::viewer::{HANDLE, OPEN, get, put};

/// The Rust version (every register kept).
pub fn close_viewed_replay() -> Result<(), Refused> {
    // Converted: `rc2k_core::race::viewer_flow::closes`.
    if rc2k_core::race::viewer_flow::closes(get(OPEN)) {
        crate::race::replay_patches::restore_replay_patches()?;
        unsafe { crate::files::handle::close(get(HANDLE)) };
        put(OPEN, 0);
    }
    Ok(())
}
