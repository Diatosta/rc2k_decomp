//! `marks::make_masks` (0x4BDE9A): in `sub_4C0F2E`, the shape sets, corner tables, rings,
//! masks and pixel tables of the two shapes, in a work block taken and freed.
//!
//! In:  nothing
//! Out: eax the spans made, ebx the work block's list, the rest as the last step
//!
//! See `marks/masks.rs`. One caller, `sub_4C0F2E`. Shadow mode runs the original alone
//! (memory cannot be taken twice). Refuses as its steps do, mid-way. Self-test: a
//! redirected copy whose ten calls go to stand-ins noting every register, against the Rust
//! version on the same stand-ins: the calls, the registers and the variables it writes.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::marks::masks::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let left = unsafe { make_masks(r, &mut game_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..left
        }
    }
    Ok(())
}
