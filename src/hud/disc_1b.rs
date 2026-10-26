//! `hud::disc_1b` (0x4B6BAC): the second dial of layout 1: its set-up (`hud::dials`), then `hud::draw_disc`.
//!
//! In:  ebp the HUD's record
//! Out: as `draw_disc` leaves them
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
        (From::Field(0x5C), rva(0x75C054), false),
        (From::Field(0x60), rva(0x75C058), false),
        (From::Field(0x64), rva(0x75C05C), false),
        (From::Var(rva(0x75C168)), rva(0x75C050), false),
        (From::Var(rva(0x75C16C)), rva(0x75C04C), false),
        (From::Var(rva(0x75C170)), rva(0x75C044), false),
        (From::Var(rva(0x75C174)), rva(0x75C048), false),
    ],
    len: 0x51,
    entry: [0xFF, 0x75, 0x5C, 0x8F, 0x05, 0x54, 0xC0, 0x75, 0x00],
    call: 0x4B,
    callee: &[(0x4B6C4E, 1)],
};

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let left = unsafe { set_up(r, &SETUP, &mut crate::hud::add_triangle::game_part_disc) }?;
    *r = Regs { esp: r.esp, ..left };
    Ok(())
}
