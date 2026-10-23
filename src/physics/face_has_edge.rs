//! `physics::face_has_edge` (0x45DA72): whether the edge (eax, ebx) is one of a face's,
//! by its corner words at esi +4 .. +0x10 (p1 .. p4): CF clear when (eax, ebx) is (p1,
//! p3), (p2, p1), (p4, p2) or (p3, p4), tested in that order; else CF.
//!
//! In:  eax, ebx the edge's two corners, esi the face
//! Out: CF; every register kept (ZF as the last compare leaves it, not read: callers
//!      `jnb`)
//!
//! Caller: `sub_45DA14` (four calls). It begins with a short jump in its first five
//! bytes: hooked by a private copy and a hand-made patch.
//! Self-test: a copy of the original on a test face, every pair of corners and some
//! that are not.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::physics::flag_result::{FlagResult, flags_entry};
use rc2k_core::physics::faces::has_edge;

unsafe fn corners(face: u32) -> [u32; 4] {
    let at = |o: u32| unsafe { ((face + o) as *const u32).read_unaligned() };
    [at(4), at(8), at(0xC), at(0x10)]
}

/// The Rust version.
// Converted: `rc2k_core::physics::faces::has_edge`.
pub unsafe fn face_has_edge(r: &Regs) -> FlagResult {
    let found = has_edge(r.eax, r.ebx, unsafe { corners(r.esi) });
    FlagResult {
        eax: r.eax,
        ecx: None,
        ebx: None,
        carry: !found,
        zero: None,
    }
}

fn rust(r: &Regs) -> Result<FlagResult, Refused> {
    Ok(unsafe { face_has_edge(r) })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    flags_entry(r, &rust)
}
