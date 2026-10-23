//! `hud::needle_1a` (0x4B6E67): the first needle of layout 1: its set-up (`hud::dials`), then `hud::draw_needle`.
//!
//! In:  ebp the HUD's record
//! Out: as `draw_needle` leaves them
//!
//! The shape through `part` (1), in the game its Rust version. Shadow mode runs the
//! original alone (it draws). Self-test: `hud::dials::test`.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, rva};
use crate::hud::dials::{From, Setup, set_up};

pub static SETUP: Setup = Setup {
    copies: &[
        (From::Field(0x10), rva(0x75C2E0), false),
        (From::Var(rva(0x75C30C)), rva(0x75C2A0), false),
        (From::Var(rva(0x75C310)), rva(0x75C2A4), false),
        (From::Var(rva(0x75C314)), rva(0x75C2B8), false),
    ],
};

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { set_up(r, &SETUP, &mut crate::hud::draw_needle::game_part_needle) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
