//! The two replay starts the review menus' items lead to (`replay_mode`, mode 1 and
//! mode 3): with a replay there (the replay-present flag not 0), the replay viewer
//! `menu::replay_viewer` with the mode.

use crate::errors::Refused;
use crate::menu::replay_viewer::{Mode, Viewed};
use crate::race::owner::Race;

/// One start in `mode`: the viewer, when there is a replay; how the view ended (`Nothing` when
/// there was none).
pub fn replay_mode(
    race: &'static Race,
    o: crate::owners::Owners<'static>,
    mode: Mode,
) -> Result<Viewed, Refused> {
    // The logic is in `rc2k_core::race::viewer_flow::views_replay`.
    if rc2k_core::race::viewer_flow::views_replay(race.shared.replay_file().replay_closed.get()) {
        return crate::menu::replay_viewer::replay_viewer(race, o, mode);
    }
    Ok(Viewed::Nothing)
}
