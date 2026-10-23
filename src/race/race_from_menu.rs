//! `race::race_from_menu` (0x485570): the menu's 3D memory tidied (`race::render3d_tidy`,
//! `race::objects_tidy`, `nullsub_6`), the textures released (`race::release_textures`),
//! then the race (`race::start_race`).
//!
//! In:  nothing
//! Out: the registers as the race leaves them
//!
//! See `race/race_end.rs`. One caller, `sub_4843EC` (a call follows). It begins with its
//! call: private copy, its five calls relocated, manual patch. Steps through
//! direct calls. Shadow mode runs the original alone (it runs the race). Self-test: a
//! copy whose four calls go to stand-ins (`nullsub_6` the real one), against the Rust
//! version on the same.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::race_end::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { race_from_menu(r) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
