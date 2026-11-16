//! The saving screen of the replay slots: the viewer's stage is tidied away (`viewer_tidy_stage`),
//! the slot is chosen on the saving screen, and the stage is loaded again (`viewer_load_stage`).
//! Called by `viewer_action` (0x16).

use crate::errors::Refused;
use crate::owners::Owners;
use crate::race::owner::Race;

/// The slot chosen.
pub fn choose_save_slot(race: &'static Race, o: Owners<'static>) -> Result<u32, Refused> {
    crate::race::viewer_tidy_stage::viewer_tidy_stage(race, o)?;
    let slot = crate::menu::replay_slot_loop::replay_slot_loop(o, true)?;
    crate::race::viewer_load_stage::viewer_load_stage(race, o)?;
    Ok(slot)
}
