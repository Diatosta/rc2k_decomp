//! Closes the viewer's replay if it is open (`replay_open` 1): the physics patches go back
//! (`restore_replay_patches`), the file `h` is closed (`files::close`) and `replay_open` is 0.
//! Called when the viewer ends, on action 0x14, when opening a replay fails and when saving the
//! last replay.

use crate::errors::Refused;
use crate::race::owner::Race;

pub(super) fn close_viewed_replay(race: &Race) -> Result<(), Refused> {
    let shared = &race.shared;
    if rc2k_core::race::viewer_flow::closes(shared.replay_file().replay_open.is_set()) {
        crate::race::replay_patches::restore_replay_patches(race)?;
        crate::files::handle::close(shared.replay_file().replay_handle.get());
        shared.replay_file().replay_open.set(false);
    }
    Ok(())
}
