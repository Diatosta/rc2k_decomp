//! `race::copy_settings_in` (0x446EBE): "COPY GRAHAM TO SHAUN": the front end's settings
//! (0x75ACxx ..) into the race's variables and the two players' blocks (copies, whole
//! numbers to floats, the brakes and others held to 0 .. 0x100, the damage bits), each
//! logged; the two players' absences cleared; the league table copied.
//!
//! In:  nothing
//! Out: eax and ebx as the steps leave them, the rest as `race::copy_league_table`
//!
//! See `race/settings_copy.rs` (the steps a table made from the disassembly by script).
//! One caller, `sub_443F0B` (`race::set_up_arcade_cars` follows; no flags read). Shadow
//! mode runs the original alone (it logs). Self-test: a copy whose 260 calls go to
//! stand-ins, against the Rust version on the same: every dword it reads set to values
//! round its tests eight ways, one and two players, the damage bits given and not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::settings_copy::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { copy_settings_in(r, &mut game_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
