//! `race::load_packed_game_map` (0x41B785): the in-race map read from its packed file.
//!
//! In:  eax = the path (`files\maps\mapxx.sq`)
//! Out: every register preserved
//!
//! See `race/game_map.rs`. One caller, `race::load_game_map`. Refuses a count of 0 or
//! past 0xFFFF and a short read not of whole pixels. Shadow mode runs the original alone
//! (it allocates and reads). Self-test: stage 21's map and a missing file, read by both,
//! compared with the pointers taken out; the map freed and the game's put back after
//! each.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_addr, rva};
use crate::race::game_map::*;
use crate::service::harness::Region;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    unsafe { load_packed_game_map(r.eax) }?;
    Ok(())
}

pub fn kept() -> Vec<Region> {
    vec![
        (game_addr(VARS.0) as u32, VARS.1),
        (game_addr(LIST_RVA) as u32, 0x40),
        (game_addr(KEY_LIST_RVA) as u32, 0x40),
        (game_addr(rva(0x93E8B0)) as u32, READ_BUFFER_LEN),
        (game_addr(PACKED_PATH_RVA) as u32, 0x40),
        crate::service::harness::at(rva(0x4F52B0)),
    ]
}
