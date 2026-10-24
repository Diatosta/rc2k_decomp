//! `race::close_viewed_replay` (0x41EB7C): the viewer's replay closed, if open
//! (`dword_5BEC88` 1): the game's physics patches back (`race::restore_replay_patches`),
//! the file `h` closed (`files::close`), `dword_5BEC88` 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`); no flags read
//!
//! Callers: `race::replay_viewer`, `race::viewer_action` (action 0x14),
//! `race::open_viewed_replay` (failing), `race::save_last_replay`, 0x444034. Its calls
//! through `part`: 1 `restore_replay_patches`, 2 `files::close` (eax the handle); in the
//! game their Rust versions. Shadow mode runs the original alone (it closes a file).
//! Self-test: a copy whose calls go to stand-ins, against the Rust version, open or not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

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
