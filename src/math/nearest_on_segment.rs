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
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::math::x87::loaded;
use crate::math::x87_run::compare_80;
use crate::physics::flow::{c0, x87_run};

/// The work block's dwords (0x71C384 .. 0x71C3D7 in the image until 2026-10-24: the
/// function's own; `sound::merge_spot_voices` wrote the ends and read the point).
const WORK_LEN: usize = 0x54 / 4;
/// t's dword (`g_segmentT`, 0x71C3B4).
const T: usize = 0x30 / 4;
/// `flt_71C3C8`: 0.0 (never written).
const SEGMENT_ZERO: u32 = 0;
/// `flt_5C5D90` (0.0) and `flt_5C5D38` (1.0).
const ZERO: u32 = 0;
const ONE: u32 = 0x3F80_0000;

// ---- Rust version ----

/// Exactly as the original (the x87 at its precision): the sound code's calls by place,
/// whose own comparisons stay exact until `sound` is converted.
pub fn nearest_on_segment(a: [u32; 3], b: [u32; 3], o: [u32; 3]) -> [u32; 3] {
    // The work block as the image laid it out (0x71C384 ..): A, B, O, the point, t, the
    // differences; the x87 reads and writes it by these offsets.
    let mut block = [0u32; WORK_LEN];
    block[..3].copy_from_slice(&a);
    block[3..6].copy_from_slice(&b);
    block[6..9].copy_from_slice(&o);
    let (mut top, mut below) = ([0u8; 10], [0u8; 10]);
    x87_run!(
        "fld dword ptr [{w} + 0x00]",
        "fsub dword ptr [{w} + 0x18]",
        "fstp dword ptr [{w} + 0x34]",
        "fld dword ptr [{w} + 0x08]",
        "fsub dword ptr [{w} + 0x20]",
        "fstp dword ptr [{w} + 0x38]",
        "fld dword ptr [{w} + 0x0C]",
        "fsub dword ptr [{w} + 0x18]",
        "fstp dword ptr [{w} + 0x3C]",
        "fld dword ptr [{w} + 0x14]",
        "fsub dword ptr [{w} + 0x20]",
        "fstp dword ptr [{w} + 0x40]",
        "fld dword ptr [{w} + 0x34]",
        "fsub dword ptr [{w} + 0x3C]",
        "fstp dword ptr [{w} + 0x4C]",
        "fld dword ptr [{w} + 0x38]",
        "fsub dword ptr [{w} + 0x40]",
        "fstp dword ptr [{w} + 0x50]",
        "fld dword ptr [{w} + 0x34]",
        "fmul dword ptr [{w} + 0x4C]",
        "fld dword ptr [{w} + 0x38]",
        "fmul dword ptr [{w} + 0x50]",
        ".byte 0xDE, 0xC1",
        "fld dword ptr [{w} + 0x3C]",
        "fmul dword ptr [{w} + 0x4C]",
        "fld dword ptr [{w} + 0x40]",
        "fmul dword ptr [{w} + 0x50]",
        ".byte 0xDE, 0xC1",
        ".byte 0xD8, 0xE9",
        "fstp tbyte ptr [{t}]",
        "fstp tbyte ptr [{u}]";
        w = block.as_mut_ptr(), t = top.as_mut_ptr(), u = below.as_mut_ptr()
    );
    // fcom flt_71C3C8; C3: equal (or unordered).
    if compare_80(top, SEGMENT_ZERO) & 0x4000 != 0 {
        block[T] = 0;
    } else {
        x87_run!(
            "fld tbyte ptr [{u}]",
            "fld tbyte ptr [{t}]",
            ".byte 0xDE, 0xF9",
            "fstp dword ptr [{w} + 0x30]";
            w = block.as_mut_ptr(), t = top.as_ptr(), u = below.as_ptr()
        );
        // `hold_low` / `hold_high` (`physics::flow`) on the block: t held to 0 .. 1.
        if c0(block[T], ZERO) {
            block[T] = loaded(ZERO);
        }
        if !c0(block[T], ONE) {
            block[T] = loaded(ONE);
        }
    }
    x87_run!(
        "fld dword ptr [{w} + 0x0C]",
        "fsub dword ptr [{w} + 0x00]",
        "fmul dword ptr [{w} + 0x30]",
        "fadd dword ptr [{w} + 0x00]",
        "fstp dword ptr [{w} + 0x24]",
        "fld dword ptr [{w} + 0x10]",
        "fsub dword ptr [{w} + 0x04]",
        "fmul dword ptr [{w} + 0x30]",
        "fadd dword ptr [{w} + 0x04]",
        "fstp dword ptr [{w} + 0x28]",
        "fld dword ptr [{w} + 0x14]",
        "fsub dword ptr [{w} + 0x08]",
        "fmul dword ptr [{w} + 0x30]",
        "fadd dword ptr [{w} + 0x08]",
        "fstp dword ptr [{w} + 0x2C]";
        w = block.as_mut_ptr()
    );
    [block[9], block[10], block[11]]
}

#[cfg(test)]
mod tests {
    use crate::moved::image_bytes;

    #[test]
    fn constants_are_the_games() {
        let word = |at: usize| u32::from_le_bytes(image_bytes(at, 4).try_into().unwrap());
        assert_eq!(word(0x71C3C8), super::SEGMENT_ZERO);
        assert_eq!(word(0x5C5D90), super::ZERO);
        assert_eq!(word(0x5C5D38), super::ONE);
    }
}
