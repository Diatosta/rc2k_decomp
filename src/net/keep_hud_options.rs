//! `net::keep_hud_options` (0x463E39): this machine's car's HUD bits (+0x220 of
//! `g_carBlocks[g_netPlayer]`) kept in `dword_75B00C` for the next race.
//!
//! In:  nothing
//! Out: esi = the car's block, eax = the bits; the rest and the flags kept
//! Caller: `net::lets_race`. Shadow mode: both run, the dword compared. Self-test: a copy
//! against the Rust version on a test block.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::{Part, p};
use crate::net::set_hud_options::{HUD_BITS, own_block};

/// The Rust version: the registers it leaves.
pub fn keep_hud_options(entry: &Regs, _: Part) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.esi = own_block();
    r.eax = unsafe { ((r.esi + 0x220) as *const u32).read_unaligned() };
    p(HUD_BITS, r.eax);
    Ok(r)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::race::viewer::regs_entry(r, &|r| {
        keep_hud_options(r, &mut crate::net::lobby::game_part)
    })
}
