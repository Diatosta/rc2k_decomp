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
use crate::physics::ground_probe_store::GROUND_PROBE;
use crate::physics::ground_store::GROUND_WORK;
use rc2k_core::physics::faces;

/// The Rust version: the ground point's side of the edge from scene point `from` to `to`
/// (their numbers).
pub fn edge_side(from: u32, to: u32) -> Result<FlagResult, crate::errors::Refused> {
    let result = |eax, carry, zero| FlagResult {
        eax,
        ecx: None,
        ebx: None,
        carry,
        zero: Some(zero),
    };
    let point = (GROUND_WORK.ground_x.get(), GROUND_WORK.ground_z.get());
    let scene = |n: u32| -> Result<(u32, u32), crate::errors::BadInput> {
        Ok((
            crate::race::scene_points::point_word(n, 0)?,
            crate::race::scene_points::point_word(n, 2)?,
        ))
    };
    Ok(match faces::edge_side(point, scene(from)?, scene(to)?) {
        None => result(1, false, false),
        Some((kept, negative)) => {
            GROUND_PROBE.product[0].set(kept as u32);
            GROUND_PROBE.product[1].set((kept >> 32) as u32);
            result(0, negative, true)
        }
    })
}
