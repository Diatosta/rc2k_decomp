//! `net::lets_race` (0x4640AA): the network race driven: `dword_63DB84` and the eight stored
//! cars' +0x74 (`g_carBlockStore.out`) 0, the HUD set (`net::set_hud_options`), the race
//! (`drive::play_race`), the HUD kept (`net::keep_hud_options`); then this machine's car
//! (`g_carBlocks[dword_63DBA0]`): eax its +0x88 (the race's time, it seems), ebx 0, or -1
//! when its +0xB0 is -1.
//!
//! In:  nothing
//! Out: eax, ebx as above, esi the car's block; the rest as the race leaves them; the flags
//!      of the last compare (no caller reads them)
//! Caller: `net::lets_rally`. Its calls through `part` (`net::lobby_calls`: the Rust
//! versions). Long-running (the race): ungated. Shadow mode runs the original alone.
//! Self-test: a copy whose calls go to stand-ins, against the Rust version, +0xB0 -1 or
//! not, on a test block.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{PLAY_RACE, Part, g, lobby_entry, p};
use crate::net::reset_cars::CAR_BLOCKS;

const SET_HUD: u32 = 0x463D3F; // net::set_hud_options
const KEEP_HUD: u32 = 0x463E39; // net::keep_hud_options
const CLEARED: usize = 0x63DB84; // dword_63DB84
const LOCAL: usize = 0x63DBA0; // dword_63DBA0
/// `g_carBlockStore.out` of the eight stored cars.
const OUTS: [usize; 8] = [
    0x70F49C, 0x71047C, 0x71145C, 0x71243C, 0x71341C, 0x7143FC, 0x7153DC, 0x7163BC,
];

/// The Rust version: the registers it leaves.
pub fn lets_race(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    p(CLEARED, 0);
    for x in OUTS {
        p(x, 0);
    }
    part(SET_HUD, &mut r)?;
    part(PLAY_RACE, &mut r)?;
    part(KEEP_HUD, &mut r)?;
    r.eax = g(LOCAL);
    r.esi = g(CAR_BLOCKS + 4 * r.eax as usize);
    let get = |offset: u32| unsafe { ((r.esi + offset) as *const u32).read_unaligned() };
    r.eax = get(0x88);
    // Converted: `rc2k_core::net::record::not_finished`.
    r.ebx = rc2k_core::net::record::not_finished(get(0xB0));
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    lobby_entry(r, lets_race)
}
