//! `race::set_up_cars` (0x43603C): race init's "cars": "gamemode" and `dword_62389C`
//! logged; both players' +0x5FC 0 and +0xA0C .. +0xA1C -1; then by the mode: 2 the
//! numbers, the times, the arcade set-up and `dword_601C68` (0x40 dwords) 0; 1, 3, 4
//! their set-ups; 0 the numbers and its set-up; others nothing.
//!
//! In:  nothing
//! Out: the registers as the mode's steps leave them (mode 2: edi past `dword_601C68`'s
//!      0x40 dwords, eax and ecx 0)
//!
//! See `race/race_cars.rs`. One caller, `sub_4462B9` (race init). Shadow mode runs the
//! original alone (it logs). Self-test: a copy whose calls note themselves, against the
//! Rust version on the same, modes 0 .. 5.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::race_cars::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { set_up_cars(r, &mut cars_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
