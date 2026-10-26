//! `net::clear_car_state` (0x463EB6): a car's block made ready for a network race: 51 of
//! its dwords (+0x64 .. +0x80, +0x2B8 .. +0x364) 0, +0x298 -1.
//!
//! In:  esi = the car's block
//! Out: every register and flag kept (only `mov`s; the flags are not compared: `call_game`
//!      does not load them)
//! Caller: `net::reset_cars`. Shadow mode: both run, the block's 0x370 bytes compared.
//! Self-test: a copy against the Rust version on a test block.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::net::lobby::Part;

/// The Rust version on the block at `block`.
pub fn clear_block(block: u32) {
    // Converted: `rc2k_core::net::cars::CLEARED`.
    for (offset, v) in rc2k_core::net::cars::CLEARED {
        unsafe { ((block + offset) as *mut u32).write_unaligned(v) };
    }
}

/// The Rust version: the registers it leaves (the entry's).
pub fn clear_car_state(entry: &Regs, _: Part) -> Result<Regs, Refused> {
    clear_block(entry.esi);
    Ok(*entry)
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    crate::race::viewer::regs_entry(r, &|r| {
        clear_car_state(r, &mut crate::net::lobby::game_part)
    })
}
