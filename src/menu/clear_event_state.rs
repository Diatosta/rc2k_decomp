//! `menu::clear_event_state` (0x486C2C): an event's state cleared at its start.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `dword_755950`, `dword_758714` and the 28 dwords `dword_758758` .. `dword_7587C4` set to 0 (the
//! results screens and the savegame read them; their meaning is not read). Callers: the event loops
//! (`sub_482382` and four more).

use crate::menu::event_store::EVENT_STATE;
use crate::menu::results_store::RESULT_FIGURES;
use crate::service::harness::Region;

const RUN_DWORDS: usize = 28;

fn regions() -> [Region; 3] {
    [
        (EVENT_STATE.stages_done.addr() as u32, 4),
        (RESULT_FIGURES.round.addr() as u32, 4),
        (RESULT_FIGURES.time_bonus[0].addr() as u32, RUN_DWORDS * 4),
    ]
}

pub fn clear_event_state() {
    for (addr, len) in regions() {
        unsafe { std::ptr::write_bytes(addr as *mut u8, 0, len) };
    }
}
