//! `marks::pong_setup` (0x4B88CB): the pong game's frame set up, as
//! `marks::squares_setup` with the texture places from `dword_769222` .. (the original
//! also loads `dword_76921E` into eax and drops it).
//!
//! In:  nothing
//! Out: eax .. edx the four places, edi `g_polygonVertices`; the rest as the device calls
//!      leave them
//!
//! Caller: `marks::run_pong`. The device calls through `part` as `squares_setup`'s.
//! Hooked through a private copy (a call at the entry). Shadow mode runs the original
//! alone (it tells the device). Self-test: a copy whose calls go to stand-ins, against
//! the Rust version.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, rva};
use crate::marks::squares_setup::{Part, game_part, setup_with};

const PLACES_RVA: usize = rva(0x769222); // dword_769222 .. dword_76922E
// g_polygonVertices

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn pong_setup(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    unsafe { setup_with(entry, PLACES_RVA, part) }
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { pong_setup(r, &mut game_part) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
