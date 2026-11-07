//! `race::choose_save_slot` (0x41E6DC): the viewer's stage tidied away
//! (`race::viewer_tidy_stage`), the saving screen of the replay slots
//! (`menu::replay_slot_loop`, eax as given: the caller's 1), the stage loaded again
//! (`race::viewer_load_stage`).
//!
//! In:  eax 1 (the saving screen)
//! Out: eax the slot chosen or -1 (the screen's; the loading keeps it); the rest as the
//!      screen leaves them
//!
//! Caller: `race::viewer_action` (0x16).

use crate::errors::Refused;
use crate::race::owner::Race;

/// The Rust version: the slot chosen (eax; -1 for none).
pub fn choose_save_slot(race: &Race) -> Result<u32, Refused> {
    // Both keep every register.
    crate::race::viewer_tidy_stage::viewer_tidy_stage(race)?;
    let slot = crate::menu::replay_slot_loop::replay_slot_loop(crate::menu::owner::menus(), true)?;
    crate::race::viewer_load_stage::viewer_load_stage(race)?;
    Ok(slot)
}
