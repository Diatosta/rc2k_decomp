//! `race::release_textures` (0x443396): `race::release_stage_textures` and
//! `scene::reset_texture_tables` (two calls and a `retn`).
//!
//! In:  nothing
//! Out: the registers as the two leave them
//!
//! See `race/race_end.rs`. Ten callers (`race::race_from_menu`, menus, films; none reads
//! flags after it). It begins with its call: private copy, both calls relocated, manual
//! patch. Shadow mode runs the original alone (it releases the textures). Self-test: a
//! copy whose calls go to stand-ins, against the Rust version on the same.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::race_end::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { release_textures(r) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
