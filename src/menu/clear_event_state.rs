//! `menu::clear_event_state` (0x486C2C): an event's state cleared at its start.
//!
//! `dword_755950`, `dword_758714` and the 28 dwords `dword_758758` .. `dword_7587C4` set to 0 (the
//! results screens and the savegame read them; their meaning is not read). Callers: the event loops
//! (`sub_482382` and four more).
use crate::menu::owner::Menus;
use crate::menu::results_store::Results;
use crate::pointers::Store;

const RUN_DWORDS: usize = 28;

pub fn clear_event_state(menus: &Menus) -> Result<(), crate::errors::BadInput> {
    menus.screens.event.stages_done.store(0, &[0; 4])?;
    menus.play.results.round.store(0, &[0; 4])?;
    menus.play.results.store(
        std::mem::offset_of!(Results, time_bonus),
        &[0; RUN_DWORDS * 4],
    )?;
    Ok(())
}
