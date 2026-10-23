//! `marks::set_up_blocks` (0x4D05BD): `init_marks`' step 14, the settings blocks' starting
//! sums measured, each block's points taken and copied, its floats set.
//!
//! In:  nothing
//! Out: as the first block's `marks::copy_block_points` leaves them, ebx its +0x4C, ebp it
//!
//! See `marks/lines.rs`. One caller, `marks::init_marks`. Refuses as
//! `marks::measure_starting_sums`, before anything is taken. Shadow mode runs the original
//! alone (memory cannot be taken twice). Manual patch with a private copy (0x136 bytes; the
//! three calls relocated): the first instruction is a call. Self-test: a redirected copy
//! whose calls go to stand-ins (the block a buffer of the test's, the other two the Rust
//! versions) on scenes of the test's, the game's variables put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::marks::lines::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let left = unsafe { set_up_blocks(r, &mut game_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..left
        }
    }
    Ok(())
}
