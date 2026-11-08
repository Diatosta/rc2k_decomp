//! `math::nearest_on_segment` (0x4633DE): the point of a segment nearest a point, seen
//! from above.
//!
//! Takes: the segment's ends A and B and the point O, each as three floats (bits).
//! Returns: the nearest point, three floats.
//! With x and z only: t = (A - O).(A - B) over |A - B|^2 (0 when that is zero), held to
//! 0 .. 1; the point = A + (B - A) t in all three. The arithmetic is the original's x87
//! (the sound code's calls by place, whose own comparisons stay exact until `sound` is
//! converted); the same maths in plain `f32` is `rc2k_core::math::geometry::nearest_on_segment`.

use crate::math::x87::loaded;
use crate::math::x87_float::{compare, ld};
use crate::physics::flow::c0;
use crate::scene::x87_mem::{Base, cells, fld, fld80, fstp, fstp80};
use std::cell::Cell;

/// The work block's dwords (the function's own variables, 0x71C384 .. 0x71C3D7 in the
/// image until 2026-10-24); the x87 code reaches them by these byte offsets.
const WORK_LEN: usize = 0x54 / 4;
/// A float's place in a three-float point.
const X: usize = 0;
const Y: usize = 1;
const Z: usize = 2;
/// The segment's ends A and B and the point O (inputs): x, y, z.
const A: [usize; 3] = [0x00, 0x04, 0x08];
const B: [usize; 3] = [0x0C, 0x10, 0x14];
const O: [usize; 3] = [0x18, 0x1C, 0x20];
/// The nearest point (output): x, y, z.
const NEAREST: [usize; 3] = [0x24, 0x28, 0x2C];
/// t along the segment (`g_segmentT`, 0x71C3B4), as a byte offset and a dword index.
const T_AT: usize = 0x30;
const T: usize = T_AT / 4;
/// A - O, B - O and A - B in x and z.
const A_MINUS_O: [usize; 2] = [0x34, 0x38];
const B_MINUS_O: [usize; 2] = [0x3C, 0x40];
const A_MINUS_B: [usize; 2] = [0x4C, 0x50];
/// `flt_71C3C8`: 0.0 (never written).
const SEGMENT_ZERO: u32 = 0;
/// `flt_5C5D90` (0.0) and `flt_5C5D38` (1.0).
const ZERO: u32 = 0;
const ONE: u32 = 0x3F80_0000;

// ---- Rust version ----

/// The differences (A - O, B - O and A - B in x and z) into the work block `w`, and the
/// 80-bit dot (A - O).(A - B) less (B - O).(A - B) into `top`, the dot (A - O).(A - B)
/// into `below` (`fstp tbyte`).
fn differences(w: impl Base, top: impl Base, below: impl Base) {
    // Only x and z count (seen from above).
    for (k, axis) in [X, Z].into_iter().enumerate() {
        fstp(w, A_MINUS_O[k], fld(w, A[axis]) - fld(w, O[axis]));
    }
    for (k, axis) in [X, Z].into_iter().enumerate() {
        fstp(w, B_MINUS_O[k], fld(w, B[axis]) - fld(w, O[axis]));
    }
    for k in 0..2 {
        fstp(w, A_MINUS_B[k], fld(w, A_MINUS_O[k]) - fld(w, B_MINUS_O[k]));
    }
    let dot =
        fld(w, A_MINUS_O[0]) * fld(w, A_MINUS_B[0]) + fld(w, A_MINUS_O[1]) * fld(w, A_MINUS_B[1]);
    let other =
        fld(w, B_MINUS_O[0]) * fld(w, A_MINUS_B[0]) + fld(w, B_MINUS_O[1]) * fld(w, A_MINUS_B[1]);
    fstp80(top, 0, dot - other);
    fstp80(below, 0, dot);
}

/// t = `below` / `top` into the work block (`fdivp`).
fn ratio(w: impl Base, top: impl Base, below: impl Base) {
    fstp(w, T_AT, fld80(below, 0) / fld80(top, 0));
}

/// The point (A + (B - A) t in x, y and z) into the work block.
fn point(w: impl Base) {
    for axis in [X, Y, Z] {
        let (a, b) = (fld(w, A[axis]), fld(w, B[axis]));
        fstp(w, NEAREST[axis], (b - a) * fld(w, T_AT) + a);
    }
}

/// Exactly as the original (the x87 at its precision): the sound code's calls by place,
/// whose own comparisons stay exact until `sound` is converted.
pub fn nearest_on_segment(a: [u32; 3], b: [u32; 3], o: [u32; 3]) -> [u32; 3] {
    // The work block as the image laid it out: A, B, O, the point, t, the differences.
    let mut block = [0u32; WORK_LEN];
    for axis in [X, Y, Z] {
        block[A[axis] / 4] = a[axis];
        block[B[axis] / 4] = b[axis];
        block[O[axis] / 4] = o[axis];
    }
    let (top, below) = ([Cell::new([0u8; 10])], [Cell::new([0u8; 10])]);
    differences(cells(&mut block), &top[..], &below[..]);
    // fcom flt_71C3C8; C3: equal (or unordered).
    if compare(fld80(&top[..], 0), ld(SEGMENT_ZERO)) & 0x4000 != 0 {
        block[T] = 0;
    } else {
        ratio(cells(&mut block), &top[..], &below[..]);
        // `hold_low` / `hold_high` (`physics::flow`) on the block: t held to 0 .. 1.
        if c0(block[T], ZERO) {
            block[T] = loaded(ZERO);
        }
        if !c0(block[T], ONE) {
            block[T] = loaded(ONE);
        }
    }
    point(cells(&mut block));
    [NEAREST[X], NEAREST[Y], NEAREST[Z]].map(|at| block[at / 4])
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions.
    reference!(ref_differences(w, t, u)
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
        "fstp tbyte ptr [{u}]");
    reference!(ref_ratio(w, t, u)
        "fld tbyte ptr [{u}]",
        "fld tbyte ptr [{t}]",
        ".byte 0xDE, 0xF9",
        "fstp dword ptr [{w} + 0x30]");
    reference!(ref_point(w)
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
        "fstp dword ptr [{w} + 0x2C]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "nearest_on_segment differences",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (0x00, In::F),
                            (0x08, In::F),
                            (0x0C, In::F),
                            (0x14, In::F),
                            (0x18, In::F),
                            (0x20, In::F),
                        ],
                        writes: &[
                            (0x34, Out::F),
                            (0x38, Out::F),
                            (0x3C, Out::F),
                            (0x40, Out::F),
                            (0x4C, Out::F),
                            (0x50, Out::F),
                        ],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::T)],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::T)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_differences(a[0], a[1], a[2]) },
                plain: |a| differences(a[0], a[1], a[2]),
            },
            Block {
                name: "nearest_on_segment ratio",
                operands: vec![
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0x30, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::T)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::T)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_ratio(a[0], a[1], a[2]) },
                plain: |a| ratio(a[0], a[1], a[2]),
            },
            Block {
                name: "nearest_on_segment point",
                operands: vec![Operand::Buffer {
                    reads: &[
                        (0x00, In::F),
                        (0x04, In::F),
                        (0x08, In::F),
                        (0x0C, In::F),
                        (0x10, In::F),
                        (0x14, In::F),
                        (0x30, In::F),
                    ],
                    writes: &[(0x24, Out::F), (0x28, Out::F), (0x2C, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_point(a[0]) },
                plain: |a| point(a[0]),
            },
        ]
    }

    #[test]
    fn nearest_on_segment_matches_the_assembly() {
        sparse(&blocks(), 5000, 208);
    }
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
