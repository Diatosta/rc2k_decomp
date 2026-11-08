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
//! Callers: the needles' set-ups (`hud::needle_*`).
//!
//! Converted 2026-10-21: the holds, the choice of the angle's way, the turn's register and
//! the corners through `rc2k_core::hud::disc` (compares of floats only: exact); the value
//! made a float and the turn's arithmetic and rounding stay the x87's (they follow its
//! control word). `dword_75C060` is written once, with the turn (the original also holds
//! the value there between, read by nothing).

use crate::errors::Refused;
use crate::hud::owner::Hud;
use crate::math::x87;
use crate::math::x87_float::{ld, to_int};
use crate::race::owner_frame::Frame;
use crate::scene::matrix3::extended;
use crate::scene::owner::scene;

/// `fld v [; fsub less]; fdiv over; fadd plus; fmul k0; fmul k1; fistp`.
fn turn_of(v: u32, less: Option<u32>, over: u32, plus: u32, k: [u32; 2]) -> u32 {
    let a = [v, less.unwrap_or(0), over, plus, k[0], k[1]];
    if less.is_some() {
        turn_less(a)
    } else {
        turn_plain(a)
    }
}

/// The Rust version (see the header).
// Converted: `rc2k_core::hud::disc::{needle_turn, needle_edx, needle_corners}`
pub fn draw_needle(frame: &Frame, hud: &Hud) -> Result<(), Refused> {
    use rc2k_core::hud::disc::{needle_corners, needle_edx, needle_turn};
    use rc2k_core::math::extended::to_f64;
    let kind = hud.block.needle_kind.get() != 0;
    let value = f32::from_bits(x87::from_int(hud.block.needle_value.get() as i32));
    let t = needle_turn(value, kind, hud.block.least.get(), |n| {
        hud.block.needle_k[n].get()
    });
    let tk = [hud.block.uv_scale_y.bits(), hud.block.turn_k.bits()];
    // The rounding follows the x87's control word: the x87's.
    let turn = turn_of(
        t.value.to_bits(),
        t.less.map(f32::to_bits),
        t.over.to_bits(),
        t.plus.to_bits(),
        tk,
    );
    hud.block.scratch.set(turn);
    let angle = needle_edx(turn);
    for (n, (x, y)) in needle_corners(
        hud.block
            .needle_corners
            .each_ref()
            .map(super::super::session::Float::bits),
    )
    .into_iter()
    .enumerate()
    {
        let vertex = &crate::d3d::vertices_store::VERTICES.vertices[n];
        let point = [extended(y), extended(x)].map(to_f64);
        crate::hud::turned_vertex::turned_vertex(hud, vertex, angle, point);
    }
    scene().drawing.scene.polygon_vertex_count.set_bits(4u32);
    crate::hud::add_polygon::add_polygon(frame, hud)
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, on the six floats:
/// `int = ((((a0 - a1) / a2) + a3) * a4) * a5`
fn turn_less(a: [u32; 6]) -> u32 {
    let [v, less, over, plus, k0, k1] = a.map(ld);
    to_int(((((v - less) / over) + plus) * k0) * k1) as u32
}

/// As `turn_less` without the second: `int = (((a0 / a2) + a3) * a4) * a5`
fn turn_plain(a: [u32; 6]) -> u32 {
    let [v, _, over, plus, k0, k1] = a.map(ld);
    to_int((((v / over) + plus) * k0) * k1) as u32
}

/// The runs over the test's operand buffers (`[a + 4 k]`, the result at `[o]`).
#[cfg(all(test, target_arch = "x86"))]
fn x87_1(a: usize, o: usize) {
    use crate::scene::x87_mem::Base;
    o.set_dword(0, turn_less(std::array::from_fn(|k| a.dword(4 * k))));
}

#[cfg(all(test, target_arch = "x86"))]
fn x87_2(a: usize, o: usize) {
    use crate::scene::x87_mem::Base;
    o.set_dword(
        0,
        turn_plain(std::array::from_fn(
            |k| if k == 1 { 0 } else { a.dword(4 * k) },
        )),
    );
}

#[cfg(all(test, target_arch = "x86"))]
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
