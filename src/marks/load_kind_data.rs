//! `marks::load_kind_data` (0x4C1235): `init_marks`' step 1: the marks' kind from the
//! settings, then the edges and the splines.
//!
//! In:  nothing
//! Out: as `marks::load_splines` leaves them
//!
//! See `marks/edges.rs`. One caller, `marks::init_marks`. Shadow mode runs the original
//! alone (its steps read files and take memory). Refuses as its steps do. Self-test: a
//! redirected copy whose four calls go to stand-ins noting every register, against the
//! Rust version on the same stand-ins (settings of the test's); put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::marks::edges::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let left = unsafe { load_kind_data(r, &mut kind_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..left
        }
    }
    Ok(())
}
