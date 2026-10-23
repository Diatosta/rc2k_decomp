//! `marks::allocate_ring` (0x4CFAC5): `init_marks`' step 13, the ring of 0x1000 entries taken and linked.
//!
//! In:  nothing
//! Out: eax 0x1000, ebx the entry 0xFFE, ecx and esi the block
//!
//! See `marks/blocks.rs`. One caller, `marks::init_marks`. Shadow mode runs the original alone (memory
//! cannot be taken twice). Self-test: a redirected copy whose calls go to stand-ins (the
//! block a buffer of the test's), the game's variables put back afterwards.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::framework::Regs;
use crate::marks::blocks::*;

/// The Rust version where Rust callers run it (no hook; a refusal is the caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    {
        let left = unsafe { allocate_ring(r, &mut game_part) }?;
        *r = Regs {
            esp: r.esp,
            eflags: r.eflags,
            ..left
        }
    }
    Ok(())
}
