//! `menu::clear_event_state` (0x486C2C): an event's state cleared at its start.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `dword_755950`, `dword_758714` and the 28 dwords `dword_758758` .. `dword_7587C4` set to 0 (the
//! results screens and the savegame read them; their meaning is not read). Callers: the event loops
//! (`sub_482382` and four more).
use crate::menu::owner::menus;
use crate::menu::results_store::Results;
use crate::pointers::GamePtr;

const RUN_DWORDS: usize = 28;

pub fn clear_event_state() -> Result<(), crate::errors::BadInput> {
    let menus = menus();
    let what = "the event\'s figures";
    GamePtr::to(&menus.screens.event.stages_done).fill(0, 4, what)?;
    GamePtr::to(&menus.play.results.round).fill(0, 4, what)?;
    GamePtr::to(&menus.play.results)
        .offset(std::mem::offset_of!(Results, time_bonus) as u32)
        .fill(0, RUN_DWORDS * 4, what)?;
    Ok(())
}
