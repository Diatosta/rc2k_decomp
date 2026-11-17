//! `race::save_last_replay`: the last race's replay saved as a replay slot: opened
//! (`race::open_viewed_replay`, 222), saved and closed (`race::close_viewed_replay`).
//!
//! Takes the slot.

use crate::errors::Refused;
use crate::race::owner::Race;
use crate::race::viewer::LAST_RACE;

/// `slot` the slot to save the last race's replay into. Whether it was saved.
pub fn save_last_replay(
    race: &Race,
    system: &crate::system::owner::System,
    slot: u32,
) -> Result<bool, Refused> {
    if crate::race::open_viewed_replay::open_viewed_replay(race, system, LAST_RACE)? {
        return Ok(false);
    }
    let saved = crate::race::save_viewed_replay::save_viewed_replay(race, system, slot)?;
    crate::race::close_viewed_replay::close_viewed_replay(race)?;
    Ok(saved)
}
