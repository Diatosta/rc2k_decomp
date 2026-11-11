//! `race::close_viewed_replay` (0x41EB7C): the viewer's replay closed, if open (`replay_open` 1):
//! the game's physics patches back (`race::restore_replay_patches`), the file `h` closed
//! (`files::close`), `replay_open` 0.
//!
//! Callers: `race::replay_viewer`, `race::viewer_action` (action 0x14),
//! `race::open_viewed_replay` (failing), `race::save_last_replay`, 0x444034.

use crate::errors::Refused;
use crate::race::owner::Race;

pub fn close_viewed_replay(race: &Race) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::viewer_flow::closes`.
    let shared = &race.shared;
    if rc2k_core::race::viewer_flow::closes(shared.replay_file.replay_open.is_set()) {
        crate::race::replay_patches::restore_replay_patches(race)?;
        crate::files::handle::close(shared.replay_file.replay_handle.get());
        shared.replay_file.replay_open.set(false);
    }
    Ok(())
}
