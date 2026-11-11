//! `race::choose_save_slot` (0x41E6DC): the viewer's stage tidied away (`race::viewer_tidy_stage`),
//! the saving screen of the replay slots, the stage loaded again (`race::viewer_load_stage`).
//!
//! Caller: `race::viewer_action` (0x16).

use crate::errors::Refused;
use crate::owners::Owners;
use crate::race::owner::Race;

/// The slot chosen.
pub fn choose_save_slot(race: &Race, o: Owners<'_>) -> Result<u32, Refused> {
    crate::race::viewer_tidy_stage::viewer_tidy_stage(race, o)?;
    let slot = crate::menu::replay_slot_loop::replay_slot_loop(true)?;
    crate::race::viewer_load_stage::viewer_load_stage(race, o)?;
    Ok(slot)
}
