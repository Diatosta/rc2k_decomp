//! `drive::finish_race_0` (0x4278C7): `mask_kind` 0 and `drive::finish_race`.
//!
//! Callers: `menu::arcade_finale`, `menu::championship_finish`, `menu::section_finish` (the
//! championship's flow). Refuses what `finish_race` refuses.

use crate::errors::Refused;
use crate::race::owner::Race;

pub fn finish_race_0(
    race: &'static Race,
    o: crate::owners::Owners<'static>,
) -> Result<(), Refused> {
    race.shared.switches.get().mask_kind.set(0);
    crate::drive::finish_race::finish_race(race, o)
}
