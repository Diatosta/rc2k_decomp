//! `math::nearest_on_segment` (0x4633DE): the point of a segment nearest a point, seen
//! from above.
//!
//! In:  the segment's ends A (`dword_71C384` .. `dword_71C38C`) and B (`dword_71C390` ..
//!      `dword_71C398`), the point O (`dword_71C39C` .. `dword_71C3A4`): floats
//! Out: `dword_71C3A8` .. `dword_71C3B0` the point; every register preserved
//! With x and z only (A - O, B - O and A - B into `flt_71C3B8` .. `flt_71C3D4`): t =
//! (A - O).(A - B) over |A - B|^2, 0 when that is equal to `flt_71C3C8` (0.0); held to
//! `flt_5C5D90` (0) .. `flt_5C5D38` (1) into `flt_71C3B4`; the point = A + (B - A) t in
//! all three. Converted 2026-10-20: `rc2k_core::math::geometry::nearest_on_segment`, plain `f32`.
//! Callers: `sound::merge_spot_voices` (and unreached code at 0x463349).
//! Hook removed 2026-10-22 (conversion phase 4): Rust callers call this directly; the
//! registers above are the original's, for reference.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::math::x87_run::compare_80;
use crate::physics::flow::{base, hold_high, hold_low, p, x87_run};
use crate::sound::turn_parts::Part;

// flt_5C5D90: 0
// flt_5C5D38: 1
const T: usize = 0x71C3B4;

// ---- Rust version ----

/// Exactly as the original (the x87 at its precision): the sound code's calls by place,
/// whose own comparisons stay exact until `sound` is converted.
pub fn nearest_on_segment_in(_: &mut Regs, _: Part) -> Result<(), Refused> {
    let b = base();
    let (mut top, mut below) = ([0u8; 10], [0u8; 10]);
    x87_run!(
        "fld dword ptr [{b} + 0x31C384]",
        "fsub dword ptr [{b} + 0x31C39C]",
        "fstp dword ptr [{b} + 0x31C3B8]",
        "fld dword ptr [{b} + 0x31C38C]",
        "fsub dword ptr [{b} + 0x31C3A4]",
        "fstp dword ptr [{b} + 0x31C3BC]",
        "fld dword ptr [{b} + 0x31C390]",
        "fsub dword ptr [{b} + 0x31C39C]",
        "fstp dword ptr [{b} + 0x31C3C0]",
        "fld dword ptr [{b} + 0x31C398]",
        "fsub dword ptr [{b} + 0x31C3A4]",
        "fstp dword ptr [{b} + 0x31C3C4]",
        "fld dword ptr [{b} + 0x31C3B8]",
        "fsub dword ptr [{b} + 0x31C3C0]",
        "fstp dword ptr [{b} + 0x31C3D0]",
        "fld dword ptr [{b} + 0x31C3BC]",
        "fsub dword ptr [{b} + 0x31C3C4]",
        "fstp dword ptr [{b} + 0x31C3D4]",
        "fld dword ptr [{b} + 0x31C3B8]",
        "fmul dword ptr [{b} + 0x31C3D0]",
        "fld dword ptr [{b} + 0x31C3BC]",
        "fmul dword ptr [{b} + 0x31C3D4]",
        ".byte 0xDE, 0xC1",
        "fld dword ptr [{b} + 0x31C3C0]",
        "fmul dword ptr [{b} + 0x31C3D0]",
        "fld dword ptr [{b} + 0x31C3C4]",
        "fmul dword ptr [{b} + 0x31C3D4]",
        ".byte 0xDE, 0xC1",
        ".byte 0xD8, 0xE9",
        "fstp tbyte ptr [{t}]",
        "fstp tbyte ptr [{u}]";
        b = b, t = top.as_mut_ptr(), u = below.as_mut_ptr()
    );
    // fcom flt_71C3C8; C3: equal (or unordered).
    if compare_80(top, crate::physics::flow::g(0x71C3C8)) & 0x4000 != 0 {
        p(T, 0);
    } else {
        x87_run!(
            "fld tbyte ptr [{u}]",
            "fld tbyte ptr [{t}]",
            ".byte 0xDE, 0xF9",
            "fstp dword ptr [{b} + 0x31C3B4]";
            b = b, t = top.as_ptr(), u = below.as_ptr()
        );
        hold_low(T, 0x5C5D90);
        hold_high(T, 0x5C5D38);
    }
    x87_run!(
        "fld dword ptr [{b} + 0x31C390]",
        "fsub dword ptr [{b} + 0x31C384]",
        "fmul dword ptr [{b} + 0x31C3B4]",
        "fadd dword ptr [{b} + 0x31C384]",
        "fstp dword ptr [{b} + 0x31C3A8]",
        "fld dword ptr [{b} + 0x31C394]",
        "fsub dword ptr [{b} + 0x31C388]",
        "fmul dword ptr [{b} + 0x31C3B4]",
        "fadd dword ptr [{b} + 0x31C388]",
        "fstp dword ptr [{b} + 0x31C3AC]",
        "fld dword ptr [{b} + 0x31C398]",
        "fsub dword ptr [{b} + 0x31C38C]",
        "fmul dword ptr [{b} + 0x31C3B4]",
        "fadd dword ptr [{b} + 0x31C38C]",
        "fstp dword ptr [{b} + 0x31C3B0]";
        b = b
    );
    Ok(())
}
