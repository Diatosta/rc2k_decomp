//! `race::copy_texture_info_tail` (0x439AE5): 0x800 bytes of the course info moved on (from +0x17E800 to +0x17F000).
//!
//! In:  nothing
//! Out: esi and edi past the bytes, ecx 0; the rest preserved
//!
//! See `race/level_init.rs`. One caller, `sub_44426F` (`race::load_stage`'s
//! "loadlevel_init"). Shadow mode runs the original alone. Self-test: a redirected copy
//! whose variables and memory are the test's and whose calls note themselves.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::race::level_init::*;

fn rust(r: &Regs, v: &Vars) -> Result<Regs, crate::errors::Refused> {
    Ok(unsafe { copy_texture_info_tail(r, v) })
}

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let left = rust(r, &Vars::game())?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..left
        }
    }
    Ok(())
}
