//! `physics::point_hits_car` (0x45E800): whether a point of an object lies inside the
//! work car's box. `dword_614110` + 1; the point (three whole numbers at esi) plus the
//! object's place (ebp +0x30 ..) plus the offset `dword_6D3B40` .. less the car's place
//! (`unk_71A10C` ..) into `dword_6D3B4C` ..; that / `flt_5C5D98` into `dword_6D3B78` ..
//! and `flt_6D3B58` .., turned by the car's heading (`physics::turn_by_heading`, ebx the
//! y) and back to whole numbers (x `flt_5C5D98`, `fistp`) into `dword_6D3090` ... Inside
//! when x is below `dword_614BC8` and not below minus it, and z below `dword_614BC0` and
//! not below minus `dword_614BC4` (each by the sign of the `cmp`, as the original). Inside
//! with the contact kind `dword_6D3624` 10, 1, 5 or 6 also: the point's x, 0, z (/
//! `flt_5C5D98`) into `unk_71A1FC` .., the car's speed's length (`unk_71A170`,
//! `physics::vector_length`) into `flt_6D3108`, `unk_71A208` raised to its size,
//! `unk_71A20C` = 1, `unk_71A1CC` = 0x32. All in `pusha` / `popa`.
//!
//! In:  esi the point, ebp the object
//! Out: CF set when inside; every register kept
//!
//! Callers: `physics::object_hits_car`, `physics::scene_point_hits_car` (both `jb`).

use crate::physics::flow::{a, g, p, x87_run};
use crate::physics::turn_by_heading::turn_by_heading;
use crate::physics::vector_length::vector_length;
use crate::scene::matrix3::rounded;
use rc2k_core::physics::contacts;

/// `dword_6D3624`: the kind of contact being looked at.
pub const KIND: usize = 0x6D3624;

/// The Rust version: whether the point at `point` of the object at `object` is inside.
// Converted: `rc2k_core::physics::contacts::{point_part, inside_box, kind_hits}` (the x87
// steps stay).
pub unsafe fn point_hits_car(point: u32, object: u32) -> bool {
    let rd = |x: u32| unsafe { (x as *const u32).read_unaligned() };
    p(0x614110, g(0x614110).wrapping_add(1));
    let mut y = 0;
    for k in 0..3u32 {
        let v = contacts::point_part(
            rd(point + 4 * k),
            rd(object + 0x30 + 4 * k),
            g(0x6D3B40 + 4 * k as usize),
            g(0x71A10C + 4 * k as usize),
        );
        p(0x6D3B4C + 4 * k as usize, v);
        if k == 1 {
            y = v;
        }
    }
    x87_run!(
        "fild dword ptr [{rc} + 0x2D3B4C]",
        "fdiv dword ptr [{rc} + 0x1C5D98]",
        "fstp dword ptr [{rc} + 0x2D3B78]",
        "fild dword ptr [{rc} + 0x2D3B50]",
        "fdiv dword ptr [{rc} + 0x1C5D98]",
        "fstp dword ptr [{rc} + 0x2D3B7C]",
        "fild dword ptr [{rc} + 0x2D3B54]",
        "fdiv dword ptr [{rc} + 0x1C5D98]",
        "fstp dword ptr [{rc} + 0x2D3B80]";
        rc = crate::race::store::asm_base()
    );
    for k in 0..3 {
        p(0x6D3B58 + 4 * k, g(0x6D3B78 + 4 * k));
    }
    unsafe { turn_by_heading(a(0x6D3B58), y) };
    x87_run!(
        "fld dword ptr [{rc} + 0x2D3B58]",
        "fmul dword ptr [{rc} + 0x1C5D98]",
        "fistp dword ptr [{rc} + 0x2D3090]",
        "fld dword ptr [{rc} + 0x2D3B5C]",
        "fmul dword ptr [{rc} + 0x1C5D98]",
        "fistp dword ptr [{rc} + 0x2D3094]",
        "fld dword ptr [{rc} + 0x2D3B60]",
        "fmul dword ptr [{rc} + 0x1C5D98]",
        "fistp dword ptr [{rc} + 0x2D3098]";
        rc = crate::race::store::asm_base()
    );
    let (x, z) = (g(0x6D3090), g(0x6D3098));
    if !contacts::inside_box(x, z, g(0x614BC8), g(0x614BC0), g(0x614BC4)) {
        return false;
    }
    if contacts::kind_hits(g(KIND)) {
        x87_run!(
            "fild dword ptr [{rc} + 0x2D3090]",
            "fdiv dword ptr [{rc} + 0x1C5D98]",
            "fstp dword ptr [{rc} + 0x31A1FC]",
            ".byte 0xD9, 0xEE", // fldz
            "fstp dword ptr [{rc} + 0x31A200]",
            "fild dword ptr [{rc} + 0x2D3098]",
            "fdiv dword ptr [{rc} + 0x1C5D98]",
            "fstp dword ptr [{rc} + 0x31A204]";
            rc = crate::race::store::asm_base()
        );
        p(0x6D3108, rounded(unsafe { vector_length(a(0x71A170)) }));
        if let Some(hit) = rc2k_core::physics::walls::raised_hit(
            f32::from_bits(g(0x6D3108)),
            f32::from_bits(g(0x71A208)),
        ) {
            p(0x71A208, hit.to_bits());
        }
        p(0x71A20C, 1);
        p(0x71A1CC, 0x32);
    }
    true
}
