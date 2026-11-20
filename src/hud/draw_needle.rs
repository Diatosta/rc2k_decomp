//! A HUD needle: its value (`HudBlock.needle_value`) held to `least` at least and made an angle
//! by the needle's constants (`needle_k`, which differ with `needle_kind`), times `uv_scale_y`
//! times `turn_k`, times -16 the turn; its four corners go through `hud::turned_vertex` and
//! `hud::add_polygon`. The value made a float and the turn's arithmetic and rounding stay the
//! x87's (they follow its control word); an unordered compare counts as below.

use crate::errors::Refused;
use crate::hud::owner::Hud;
use crate::hud::pen::Pen;
use crate::math::matrix3::extended;
use crate::math::x87;
use crate::math::x87_float::{ld, to_int};
use crate::race::owner_frame::Frame;

/// ((v [- less]) / over + plus) x k0 x k1 as a whole number, rounded as the x87 rounds.
fn turn_of(v: f32, less: Option<f32>, over: f32, plus: f32, k: [f32; 2]) -> u32 {
    let a = [v, less.unwrap_or(0.0), over, plus, k[0], k[1]];
    if less.is_some() {
        turn_less(a)
    } else {
        turn_plain(a)
    }
}

pub fn draw_needle(frame: &Frame, hud: &mut Hud, pen: Pen) -> Result<(), Refused> {
    use rc2k_core::hud::disc::{needle_angle, needle_corners, needle_turn};
    use rc2k_core::math::extended::to_f64;
    let kind = hud.block.needle_kind != 0;
    let value = x87::from_int(hud.block.needle_value as i32);
    let t = needle_turn(value, kind, hud.block.least, |n| hud.block.needle_k[n]);
    let tk = [hud.block.uv_scale_y, hud.block.turn_k];
    // The rounding follows the x87's control word: the x87's.
    let turn = turn_of(t.value, t.less, t.over, t.plus, tk);
    hud.block.fistp_scratch = turn;
    let angle = needle_angle(turn);
    for (n, (x, y)) in needle_corners(hud.block.needle_corners.map(f32::to_bits))
        .into_iter()
        .enumerate()
    {
        let vertex = &pen.vertices.vertices[n];
        let point = [extended(y), extended(x)].map(to_f64);
        crate::hud::turned_vertex::turned_vertex(pen.engine_tables, hud, vertex, angle, point);
    }
    pen.polygons.polygon_vertex_count.set(4u32);
    crate::hud::add_polygon::add_polygon(frame, hud, pen)
}

/// The recorded x87 run, in plain Rust over `X`, on the six floats:
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
fn x87_1(a: impl crate::math::x87_mem::Base, o: impl crate::math::x87_mem::Base) {
    o.set_dword(
        0,
        turn_less(std::array::from_fn(|k| f32::from_bits(a.dword(4 * k)))),
    );
}

#[cfg(test)]
fn x87_2(a: impl crate::math::x87_mem::Base, o: impl crate::math::x87_mem::Base) {
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

    // The recorded run, one function per run.

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
