//! `marks::load_edges` (0x4C0F2E): `init_marks`' step 1, second half: the kinds' records
//! scaled once, the kind's edge picture read and packed, the shape tilt, then the masks
//! and the two item kinds.
//!
//! In:  nothing
//! Out: as `marks::set_up_squares` leaves them, the rest as the earlier steps
//!
//! See `marks/edges.rs`. One caller, `sub_4C1235`. Shadow mode runs the original alone (it
//! reads a file and takes memory). Refuses a once-only mark neither 0 nor `loc_4C0FFD`, and
//! a picture that is not read. Self-test: a redirected copy whose five calls go to
//! stand-ins noting every register (the picture a pattern of the test's, settings of the
//! test's), against the Rust
//! version on the same stand-ins, with the mark 0 and set; everything put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::{Regs, game_addr};
use crate::marks::edges::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    let resume = game_addr(RESUME_RVA) as u32;
    {
        let left = unsafe { load_edges(r, resume) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..left
        }
    }
    Ok(())
}
