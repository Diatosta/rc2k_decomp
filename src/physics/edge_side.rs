//! `physics::edge_side` (0x4425EF): which side of the edge from the point at ebp to the
//! point at edi (whole-number x at +0, z at +8) the point (`dword_639824`,
//! `dword_63982C`) lies: (X - x0)(z1 - z0) - (Z - z0)(x1 - x0) in 64 bits (`imul`, then
//! `sub`/`sbb`), the second product kept in `qword_620460`.
//!
//! In:  ebp, edi the edge's ends
//! Out: eax 1 with ZF and CF clear when the ends have the same x and z (nothing stored);
//!      else eax 0, ZF set, CF set when the difference is negative. The rest kept (ebx,
//!      edx pushed and popped). The callers read ZF (`jnz`) then CF (`jb`).
//!
//! Callers (8): `sub_442006`, the face under a point (each edge of a triangle or a quad).
//! Shadow mode: `physics/flag_result.rs`; both versions store the same `qword_620460` (a
//! scratch value), which only the self-test compares.
//! Self-test: a copy of the original on a test point and qword, edges by lot (some with
//! the same ends, some large enough to overflow 32 bits).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{Regs, game_addr, rva};
use crate::physics::flag_result::{FlagResult, flags_entry};
use rc2k_core::physics::faces;

/// The point's and the scratch value's addresses: the game's, or a test's.
#[derive(Clone, Copy)]
pub struct EdgePlaces {
    pub x: u32,
    pub z: u32,
    pub product: u32,
}

impl EdgePlaces {
    pub fn game() -> Self {
        let p = |x: usize| game_addr(rva(x)) as u32;
        Self {
            x: p(0x639824),
            z: p(0x63982C),
            product: p(0x620460),
        }
    }
}

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// The Rust core; `eax` what eax held at entry is not needed (it is always set).
// Converted: `rc2k_core::physics::faces::edge_side`.
pub unsafe fn edge_side_at(p: &EdgePlaces, from: u32, to: u32) -> FlagResult {
    let result = |eax, carry, zero| FlagResult {
        eax,
        ecx: None,
        ebx: None,
        carry,
        zero: Some(zero),
    };
    unsafe {
        let point = (at(p.x), at(p.z));
        match faces::edge_side(point, (at(from), at(from + 8)), (at(to), at(to + 8))) {
            None => result(1, false, false),
            Some((kept, negative)) => {
                (p.product as *mut i64).write_unaligned(kept);
                result(0, negative, true)
            }
        }
    }
}

/// The Rust version.
pub unsafe fn edge_side(from: u32, to: u32) -> FlagResult {
    unsafe { edge_side_at(&EdgePlaces::game(), from, to) }
}

fn rust(r: &Regs) -> Result<FlagResult, Refused> {
    Ok(unsafe { edge_side(r.ebp, r.edi) })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    flags_entry(r, &rust)
}
