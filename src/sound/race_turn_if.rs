//! `sound::race_turn_if`: `sound::race_turn` while the race's sounds are on.

use crate::errors::Refused;

/// `sound::race_turn` while the race's sounds are on: true when it ran.
pub(super) fn race_turn_if(
    o: crate::owners::Owners<'static>,
    race: &crate::race::owner::Race,
) -> Result<bool, Refused> {
    if o.sound.race.sounds_on.get().get() == 0 {
        return Ok(false);
    }
    crate::sound::race_turn::race_turn(o, race)?;
    Ok(true)
}
