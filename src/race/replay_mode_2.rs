//! `race::replay_mode_2`: the replay viewer in mode 2 (a saved replay chosen first), with no check
//! of a replay there (unlike modes 1 and 3).

use crate::errors::Refused;
use crate::race::owner::Race;

pub fn replay_mode_2(
    race: &'static Race,
    o: crate::owners::Owners<'static>,
) -> Result<(), Refused> {
    crate::race::replay_viewer::replay_viewer(race, o, crate::race::replay_viewer::Mode::Saved)
        .map(|_| ())
}
