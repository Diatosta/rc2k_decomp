//! `race::load_level` (0x4455FC): the course data read into the course memory and set up
//! (locacurrcourse's "loadlevel").
//!
//! In:  nothing
//! Out: eax and edx the cdat digits, ecx 0, ebx the face count, esi the last face, edi past
//!      the header; ebp preserved
//!
//! See `race/course_data.rs`. One caller, `sub_4394CB` (locacurrcourse). A failed read is
//! fatal error 0x80650210 (no message), raised from Rust. Shadow mode runs the original
//! alone (it reads and logs). Self-test: a redirected copy whose eleven calls note
//! themselves (the read given or failed; its jump to the fatal stub landed), against the
//! Rust version noting the same.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::course_data::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let g = unsafe { load_level(r) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..g
        }
    }
    Ok(())
}
