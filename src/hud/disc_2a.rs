//! `hud::disc_2a` (0x4B6AA4): the first dial of layout 2 (`sub_4B3B64`): its set-up (`hud::dials`), then `hud::draw_disc`.
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
        (From::Field(0x34), rva(0x75C054), false),
        (From::Field(0x38), rva(0x75C058), false),
        (From::Field(0x3C), rva(0x75C05C), false),
        (From::Var(rva(0x75C134)), rva(0x75C050), false),
        (From::Var(rva(0x75C138)), rva(0x75C04C), false),
        (From::Var(rva(0x75C13C)), rva(0x75C044), false),
        (From::Var(rva(0x75C140)), rva(0x75C048), false),
        (From::Var(rva(0x75C144)), rva(0x75C048), true),
    ],
};
