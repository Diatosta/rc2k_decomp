//! `hud::draw_needle` (0x4B6ECD): a HUD needle: its value (`HudBlock.needle_value`, as a float through
//! `HudBlock.fistp_scratch`) held to `HudBlock.least` at least; with `HudBlock.needle_kind` held to `HudBlock.needle_k[3]` at most
//! and made an angle over `HudBlock.needle_k[4]` plus `HudBlock.needle_k[0]`; without, held to `HudBlock.needle_k[6]`, and below
//! `HudBlock.needle_k[2]` over `HudBlock.needle_k[5]` plus `HudBlock.needle_k[0]`, else less `HudBlock.needle_k[2]` over `HudBlock.needle_k[4]` plus
//! `HudBlock.needle_k[1]`; times `HudBlock.uv_scale_y` times `HudBlock.turn_k` (`fistp`), times -16 the turn; the needle's
//! four corners (x `HudBlock.needle_corners[0]` / `HudBlock.needle_corners[2]`, y `HudBlock.needle_corners[1]` / `HudBlock.needle_corners[3]`) by
//! `hud::turned_vertex`, then `hud::add_polygon` on four. (The holds by the compares' C0, an
//! unordered compare counting as below.)
//!
//! Callers: the needles' set-ups (`hud::needle_*`).
//!
//! The value made a float and the turn's arithmetic and rounding stay the x87's (they follow its
//! control word). `HudBlock.fistp_scratch` is written once, with the turn (the original also holds the value
//! there between, read by nothing).

use crate::errors::Refused;
use crate::hud::owner::{Hud, Pen};
use crate::math::x87;
use crate::math::x87_float::{ld, to_int};
use crate::race::owner_frame::Frame;
use crate::scene::matrix3::extended;

/// `fld v [; fsub less]; fdiv over; fadd plus; fmul k0; fmul k1; fistp`.
fn turn_of(v: f32, less: Option<f32>, over: f32, plus: f32, k: [f32; 2]) -> u32 {
    let a = [v, less.unwrap_or(0.0), over, plus, k[0], k[1]];
    if less.is_some() {
        turn_less(a)
    } else {
        turn_plain(a)
    }
}

// Converted: `rc2k_core::hud::disc::{needle_turn, needle_angle, needle_corners}`
pub fn draw_needle(frame: &Frame, hud: &Hud, pen: Pen) -> Result<(), Refused> {
    use rc2k_core::hud::disc::{needle_angle, needle_corners, needle_turn};
    use rc2k_core::math::extended::to_f64;
    let kind = hud.block.needle_kind.get() != 0;
    let value = x87::from_int(hud.block.needle_value.get() as i32);
    let t = needle_turn(value, kind, hud.block.least.get(), |n| {
        hud.block.needle_k[n].get()
    });
    let tk = [hud.block.uv_scale_y.get(), hud.block.turn_k.get()];
    // The rounding follows the x87's control word: the x87's.
    let turn = turn_of(t.value, t.less, t.over, t.plus, tk);
    hud.block.fistp_scratch.set(turn);
    let angle = needle_angle(turn);
    for (n, (x, y)) in needle_corners(
        hud.block
            .needle_corners
            .each_ref()
            .map(crate::session::PlainFloat::bits),
    )
    .into_iter()
    .enumerate()
    {
        let vertex = &pen.vertices.vertices[n];
        let point = [extended(y), extended(x)].map(to_f64);
        crate::hud::turned_vertex::turned_vertex(hud, vertex, angle, point);
    }
    pen.polygons.polygon_vertex_count.set(4u32);
    crate::hud::add_polygon::add_polygon(frame, hud, pen)
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, on the six floats:
/// `int = ((((a0 - a1) / a2) + a3) * a4) * a5`
fn turn_less(a: [f32; 6]) -> u32 {
    let [v, less, over, plus, k0, k1] = a.map(ld);
    to_int(((((v - less) / over) + plus) * k0) * k1) as u32
}

/// As `turn_less` without the second: `int = (((a0 / a2) + a3) * a4) * a5`
fn turn_plain(a: [f32; 6]) -> u32 {
    let [v, _, over, plus, k0, k1] = a.map(ld);
    to_int((((v / over) + plus) * k0) * k1) as u32
}

/// The runs over the test's operand buffers (`[a + 4 k]`, the result at `[o]`).
#[cfg(test)]
fn x87_1(a: impl crate::scene::x87_mem::Base, o: impl crate::scene::x87_mem::Base) {
    o.set_dword(
        0,
        turn_less(std::array::from_fn(|k| f32::from_bits(a.dword(4 * k)))),
    );
}

#[cfg(test)]
fn x87_2(a: impl crate::scene::x87_mem::Base, o: impl crate::scene::x87_mem::Base) {
    o.set_dword(
        0,
        turn_plain(std::array::from_fn(|k| {
            if k == 1 {
                0.0
            } else {
                f32::from_bits(a.dword(4 * k))
            }
        })),
    );
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::vectors::{Block, In, Operand, Out, sparse};

    // The original's instructions, one function per run.

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
                plain: |a| x87_1(a[0].cells(), a[1].cells()),
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
                plain: |a| x87_2(a[0].cells(), a[1].cells()),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("hud.draw_needle", &blocks(), 1200, 100);
    }
}
