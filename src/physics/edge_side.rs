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

use crate::physics::flag_result::FlagResult;
use crate::pointers::GamePtr;
use rc2k_core::physics::faces;

/// The point's and the scratch value's addresses: the game's, or a test's.
#[derive(Clone, Copy)]
pub struct EdgePlaces {
    pub x: u32,
    pub z: u32,
    pub product: u32,
}

impl EdgePlaces {}

unsafe fn at(a: u32) -> u32 {
    crate::pointers::read_or_end::<u32>(a, "physics::edge_side's operand")
}

/// The Rust core; `eax` what eax held at entry is not needed (it is always set).
// Converted: `rc2k_core::physics::faces::edge_side`.
pub unsafe fn edge_side_at(
    p: &EdgePlaces,
    from: u32,
    to: u32,
) -> Result<FlagResult, crate::errors::Refused> {
    let result = |eax, carry, zero| FlagResult {
        eax,
        ecx: None,
        ebx: None,
        carry,
        zero: Some(zero),
    };
    unsafe {
        let point = (at(p.x), at(p.z));
        let scene = |a: u32| -> Result<(u32, u32), crate::errors::BadInput> {
            Ok((
                GamePtr::new(a).read::<u32>("a scene point's x")?,
                GamePtr::new(a).offset(8).read::<u32>("a scene point's z")?,
            ))
        };
        Ok(match faces::edge_side(point, scene(from)?, scene(to)?) {
            None => result(1, false, false),
            Some((kept, negative)) => {
                (p.product as *mut i64).write_unaligned(kept);
                result(0, negative, true)
            }
        })
    }
}
