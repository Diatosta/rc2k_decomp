//! The two replay starts the review menus' items lead to (`race::replay_mode_1`,
//! `race::replay_mode_3`): with a replay there (`dword_5BEC84` not 0), the replay viewer
//! `race::replay_viewer` with eax the mode.

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::race::replay_viewer::Mode;

/// One start in `mode` (the original's eax): the viewer, when there is a replay. (The
/// original leaves eax `mode` with a replay, else keeps every register; nothing reads
/// them.)
pub fn replay_mode(race: &Race, mode: Mode) -> Result<(), Refused> {
    // Converted: `rc2k_core::race::viewer_flow::views_replay`.
    if rc2k_core::race::viewer_flow::views_replay(race.shared.replay_file.replay_closed.bits()) {
        crate::race::replay_viewer::replay_viewer(race, mode)?;
    }
    Ok(())
}
