//! `race::replay_mode_2` (0x41CD9D): the replay viewer in mode 2 (a saved replay chosen first),
//! with no check of a replay there (unlike `replay_mode_1` / `_3`).
//!
//! Caller: `menu::replay_item_c` (a menu item).

use crate::errors::Refused;
use crate::race::owner::Race;

pub fn replay_mode_2(
    race: &'static Race,
    o: crate::owners::Owners<'static>,
) -> Result<(), Refused> {
    crate::race::replay_viewer::replay_viewer(race, o, crate::race::replay_viewer::Mode::Saved)
}
