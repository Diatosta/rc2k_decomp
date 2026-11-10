//! `race::replay_mode_2` (0x41CD9D): the replay viewer with eax 2 (a saved replay chosen first),
//! with no check of a replay there (unlike `replay_mode_1` / `_3`).
//!
//! Caller: `menu::replay_item_c` (a menu item).

use crate::errors::Refused;
use crate::race::owner::Race;

pub fn replay_mode_2(race: &Race) -> Result<(), Refused> {
    crate::race::replay_viewer::replay_viewer(race, crate::race::replay_viewer::Mode::Saved)
}
