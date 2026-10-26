//! `hud::draw_needle` (0x4B6ECD): a HUD needle: its value (`dword_75C2E0`, as a float
//! through `dword_75C060`) held to `flt_75C018` at least; with `dword_75BFAC` held to
//! `flt_75C2C8` at most and made an angle over `flt_75C2CC` plus `flt_75C2BC`; without,
//! held to `flt_75C2D4`, and below `flt_75C2C4` over `flt_75C2D0` plus `flt_75C2BC`, else
//! less `flt_75C2C4` over `flt_75C2CC` plus `flt_75C2C0`; times `flt_75C010` times
//! `flt_75C014` (`fistp`), times -16 the turn; the needle's four corners (x `flt_75C2A8` /
//! `flt_75C2B0`, y `flt_75C2AC` / `flt_75C2B4`) by `hud::turned_vertex`, then
//! `hud::add_polygon` on four. (The holds by the compares' C0, an unordered compare
//! counting as below.)
//!
//! In:  nothing
//! Out: edx the turn, then as `add_polygon` leaves them
//!
//! Callers: the needles' set-ups (`hud::needle_*`).
//!
//! Converted 2026-10-21: the holds, the choice of the angle's way, the turn's register and
//! the corners through `rc2k_core::hud::disc` (compares of floats only: exact); the value
//! made a float and the turn's arithmetic and rounding stay the x87's (they follow its
//! control word). `dword_75C060` is written once, with the turn (the original also holds
//! the value there between, read by nothing).

use crate::errors::Refused;
use crate::hud::store::HUD;
use crate::math::x87;
use crate::scene::matrix3::extended;
use crate::scene::x87_mem::{fistp, fld};

/// `fld v [; fsub less]; fdiv over; fadd plus; fmul k0; fmul k1; fistp`.
fn turn_of(v: u32, less: Option<u32>, over: u32, plus: u32, k: [u32; 2]) -> u32 {
    let (a, mut out) = ([v, less.unwrap_or(0), over, plus, k[0], k[1]], 0u32);
    if less.is_some() {
        x87_1(a.as_ptr() as usize, (&raw mut out) as usize);
    } else {
        x87_2(a.as_ptr() as usize, (&raw mut out) as usize);
    }
    out
}

/// The Rust version (see the header).
// Converted: `rc2k_core::hud::disc::{needle_turn, needle_edx, needle_corners}`
pub fn draw_needle() -> Result<(), Refused> {
    use rc2k_core::hud::disc::{needle_corners, needle_edx, needle_turn};
    use rc2k_core::math::extended::to_f64;
    let kind = HUD.needle_kind.get() != 0;
    let value = f32::from_bits(x87::from_int(HUD.needle_value.get() as i32));
    let t = needle_turn(value, kind, HUD.least.get(), |n| HUD.needle_k[n].get());
    let tk = [HUD.uv_scale_y.bits(), HUD.turn_k.bits()];
    // The rounding follows the x87's control word: the x87's.
    let turn = turn_of(
        t.value.to_bits(),
        t.less.map(f32::to_bits),
        t.over.to_bits(),
        t.plus.to_bits(),
        tk,
    );
    HUD.scratch.set(turn);
    let angle = needle_edx(turn);
    for (n, (x, y)) in needle_corners(HUD.needle_corners.each_ref().map(|c| c.bits()))
        .into_iter()
        .enumerate()
    {
        let vertex = crate::d3d::vertices_store::piece_addr() as u32 + 0x20 * n as u32;
        let point = [extended(y), extended(x)].map(to_f64);
        crate::hud::turned_vertex::turned_vertex(vertex, angle, point)?;
    }
    crate::scene::draw_store::DRAW
        .polygon_vertex_count
        .set_bits(4u32);
    crate::hud::add_polygon::add_polygon()
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [o + 0] = (((([a + 0] - [a + 4]) / [a + 8]) + [a + 0xC]) * [a + 0x10]) * [a + 0x14]`
fn x87_1(a: usize, o: usize) {
    unsafe {
        let a = a as *const u8;
        let o = o as *mut u8;
        let t0 = fld(a, 0);
        let t1 = fld(a, 4);
        let t2 = t0 - t1;
        let t3 = fld(a, 8);
        let t4 = t2 / t3;
        let t5 = fld(a, 0xC);
        let t6 = t4 + t5;
        let t7 = fld(a, 0x10);
        let t8 = t6 * t7;
        let t9 = fld(a, 0x14);
        let t10 = t8 * t9;
        fistp(o, 0, t10);
    }
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [o + 0] = ((([a + 0] / [a + 8]) + [a + 0xC]) * [a + 0x10]) * [a + 0x14]`
fn x87_2(a: usize, o: usize) {
    unsafe {
        let a = a as *const u8;
        let o = o as *mut u8;
        let t0 = fld(a, 0);
        let t1 = fld(a, 8);
        let t2 = t0 / t1;
        let t3 = fld(a, 0xC);
        let t4 = t2 + t3;
        let t5 = fld(a, 0x10);
        let t6 = t4 * t5;
        let t7 = fld(a, 0x14);
        let t8 = t6 * t7;
        fistp(o, 0, t8);
    }
}

#[cfg(test)]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(a, o) "fld dword ptr [{a}]", "fsub dword ptr [{a} + 4]", "fdiv dword ptr [{a} + 8]", "fadd dword ptr [{a} + 12]", "fmul dword ptr [{a} + 16]", "fmul dword ptr [{a} + 20]", "fistp dword ptr [{o}]");
    reference!(ref_1(a, o) "fld dword ptr [{a}]", "fdiv dword ptr [{a} + 8]", "fadd dword ptr [{a} + 12]", "fmul dword ptr [{a} + 16]", "fmul dword ptr [{a} + 20]", "fistp dword ptr [{o}]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "draw_needle line 51",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (0, In::F),
                            (4, In::F),
                            (8, In::F),
                            (0xC, In::F),
                            (0x10, In::F),
                            (0x14, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::I)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0], a[1]) },
                plain: |a| x87_1(a[0], a[1]),
            },
            Block {
                name: "draw_needle line 55",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (0, In::F),
                            (8, In::F),
                            (0xC, In::F),
                            (0x10, In::F),
                            (0x14, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::I)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0], a[1]) },
                plain: |a| x87_2(a[0], a[1]),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
