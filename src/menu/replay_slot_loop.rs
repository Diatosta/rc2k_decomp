//! `menu::replay_slot_loop`: the replay slots' screen: the loading one (`menu::load_replay_loop`)
//! or else the saving one (`menu::save_replay_loop`); each leaves the slot chosen, -1 for none.
//!
//! Callers: `race::replay_viewer` (modes 2, 3), `race::viewer_action`, `race::choose_save_slot`.

use crate::errors::{BadInput, Refused};
use crate::menu::screen_loop::{Entry, run_screen};

/// The slot chosen (-1 for none), on the saving screen when `saving`, else on the loading one.
pub fn replay_slot_loop(o: crate::owners::Owners<'static>, saving: bool) -> Result<u32, Refused> {
    let screen = if saving {
        &crate::menu::save_replay_loop::SCREEN
    } else {
        &crate::menu::load_replay_loop::SCREEN
    };
    let slot = { run_screen(screen, &Entry::new(o)) }?;
    // Both screens leave the slot in their `result`.
    Ok(slot.ok_or(BadInput("a replay slot screen without its result"))?)
}
