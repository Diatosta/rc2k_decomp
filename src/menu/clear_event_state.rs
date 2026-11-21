//! An event's state cleared at its start: the stages done, the round and every player's results
//! figures (the order of the places is kept).

use crate::menu::owner::Menus;

pub(super) fn clear_event_state(menus: &Menus) -> Result<(), crate::errors::BadInput> {
    menus.screens.event.write().stages_done = 0;
    menus.play.results.clear_for_event();
    Ok(())
}
