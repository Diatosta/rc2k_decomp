//! `mask_kind` 0 and `drive::finish_race`.
//!
//! Refuses what `finish_race` refuses.

use crate::errors::Refused;
use crate::race::owner::Race;

pub fn finish_race_0(
    race: &'static Race,
    o: crate::owners::Owners<'static>,
) -> Result<(), Refused> {
    race.shared.switches.borrow_mut().mask_kind = 0;
    crate::drive::finish_race::finish_race(race, o)
}
