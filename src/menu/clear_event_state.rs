//! `menu::clear_event_state` (0x486C2C): an event's state cleared at its start.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `dword_755950`, `dword_758714` and the 28 dwords `dword_758758` .. `dword_7587C4` set to 0 (the
//! results screens and the savegame read them; their meaning is not read). Callers: the event loops
//! (`sub_482382` and four more).

use crate::framework::{game_addr, rva};
use crate::service::harness::{Region, at};

const FIRST_RVA: usize = rva(0x755950); // dword_755950
const SECOND_RVA: usize = rva(0x758714); // dword_758714
const RUN_RVA: usize = rva(0x758758); // dword_758758 .. dword_7587C4
const RUN_DWORDS: usize = 28;

fn regions() -> [Region; 3] {
    [
        at(FIRST_RVA),
        at(SECOND_RVA),
        (game_addr(RUN_RVA) as u32, RUN_DWORDS * 4),
    ]
}

pub fn clear_event_state() {
    for (addr, len) in regions() {
        unsafe { std::ptr::write_bytes(addr as *mut u8, 0, len) };
    }
}
