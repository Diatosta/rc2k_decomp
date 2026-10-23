//! `race::load_stage` (0x444040): the stage's whole loading ("SHAUNINITEVERYTHING"), see
//! `race/stage_loading.rs`.
//!
//! In:  nothing
//! Out: eax = `g_sceneClock`, carry clear (`clc`); the rest as race init and the clock
//!      leave them
//!
//! Callers: seven (0x41CE94, 0x443F68, 0x443FC3, 0x444020, 0x459304, 0x4636E3,
//! 0x463D1F); each calls on at once (no flags read; carry cleared anyway). Its steps are
//! the Rust versions but race init (`sub_4462B9`), called in the game (section 15) until
//! it is ported; a step whose Rust version refuses runs in the game through its hook. It
//! begins with its call: private copy, its 19 calls relocated, manual patch. Shadow mode
//! runs the original alone (it logs and loads the stage). Self-test: a copy whose calls go
//! to stand-ins that note them, its ten references to `g_logIndent` the test's, against
//! the Rust version on the same: replay on and off, `dword_64BB18` on and off.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, rva};
use crate::race::stage_loading::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let indent = crate::framework::game_addr(rva(0x4E00A0)) as u32; // g_logIndent
    {
        let g = unsafe { load_stage(r, indent, &mut game_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        };
        r.set_carry(false);
    }
    Ok(())
}
