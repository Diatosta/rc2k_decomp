//! An event's state cleared at its start: the stages done, the round and every player's results
//! figures (the order of the places is kept).

use crate::cells::Store;
use crate::menu::owner::Menus;

pub fn clear_event_state(menus: &Menus) -> Result<(), crate::errors::BadInput> {
    menus.screens.event.stages_done.store(0, &[0; 4])?;
    menus.play.results.clear_for_event();
    Ok(())
}
