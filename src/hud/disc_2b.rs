//! `hud::disc_2b` (0x4B6B0A): the second dial of layout 2: its set-up (`hud::dials`), then `hud::draw_disc`.
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

use crate::framework::rva;
use crate::hud::dials::{From, Setup};

pub static SETUP: Setup = Setup {
    copies: &[
        (From::Field(0x5C), rva(0x75C054), false),
        (From::Field(0x60), rva(0x75C058), false),
        (From::Field(0x64), rva(0x75C05C), false),
        (From::Var(rva(0x75C148)), rva(0x75C050), false),
        (From::Var(rva(0x75C14C)), rva(0x75C04C), false),
        (From::Var(rva(0x75C150)), rva(0x75C044), false),
        (From::Var(rva(0x75C154)), rva(0x75C048), false),
    ],
};
