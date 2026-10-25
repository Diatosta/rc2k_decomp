//! The two replay starts the review menus' items lead to (`race::replay_mode_1`,
//! `race::replay_mode_3`): with a replay there (`dword_5BEC84` not 0), the replay viewer
//! `race::replay_viewer` (0x41CF0C) with eax the mode.

use crate::errors::Refused;
use crate::menu::screen_loop::Carried;
use crate::race::replay_file_store::REPLAY_FILE;

/// One start in `mode` (the original's eax): the viewer, when there is a replay. `c` the
/// carried values the item gets. (The original leaves eax `mode` with a replay, else keeps
/// every register; nothing reads them.)
pub unsafe fn replay_mode(c: &Carried, mode: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::viewer_flow::views_replay`.
    if rc2k_core::race::viewer_flow::views_replay(REPLAY_FILE.replay_closed.bits()) {
        crate::race::replay_viewer::replay_viewer(c, mode)?;
    }
    Ok(())
}
