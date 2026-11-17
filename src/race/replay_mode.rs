//! The two replay starts the review menus' items lead to (`race::replay_mode_1`,
//! `race::replay_mode_3`): with a replay there (the replay-present flag not 0), the replay viewer
//! `race::replay_viewer` with the mode.

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::race::replay_viewer::{Mode, Viewed};

/// One start in `mode`: the viewer, when there is a replay; how the view ended (`Nothing` when
/// there was none).
pub fn replay_mode(
    race: &'static Race,
    o: crate::owners::Owners<'static>,
    mode: Mode,
) -> Result<Viewed, Refused> {
    // Converted: `rc2k_core::race::viewer_flow::views_replay`.
    if rc2k_core::race::viewer_flow::views_replay(race.shared.replay_file().replay_closed.get()) {
        return crate::race::replay_viewer::replay_viewer(race, o, mode);
    }
    Ok(Viewed::Nothing)
}
