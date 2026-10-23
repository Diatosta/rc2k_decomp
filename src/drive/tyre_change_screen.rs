//! `drive::tyre_change_screen` (0x445047): the tyre-change screen while a tyre change is
//! under way (the first block's +0x26C, `g_carBlockStore`, not 0 or -1):
//! `drive::tyre_screen` for the mode `dword_6238A4`, car 0.
//!
//! In:  nothing
//! Out: eax the mode, esi 0 when drawn; the rest kept; no flags read
//!
//! Caller: `sub_45429B` (the race's frame, stream A's).
//!
//! Shadow mode runs the original alone (it draws). Self-test: a copy whose calls go to
//! stand-ins noting them (`drive/tyre_parts.rs`), against the Rust version with the same.

use crate::drive::tyre_parts::*;
use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::flow::g;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { tyre_change_screen(r) }?;
        let esp = r.esp;
        *r = Regs {
            eflags: r.eflags,
            ..g
        };
        r.esp = esp;
    }
    Ok(())
}

/// The Rust version, the screen through `part`.
// Converted: `rc2k_core::drive::race_ticks::tyre_change_shown`.
pub unsafe fn tyre_change_screen(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    if !rc2k_core::drive::race_ticks::tyre_change_shown(g(0x70F3EC + 0x26C)) {
        return Ok(r);
    }
    r.eax = g(0x6238A4);
    r.esi = 0;
    let mut s = r;
    crate::drive::tyre_parts::game_part(SCREEN, &mut s)?;
    Ok(r)
}
