//! A vertex of the HUD's disc from its ring entry (0 the centre .. 0x10): the texture place; the
//! place; colour white under the quad record's alpha, specular 0. Refuses an entry past 0x10,
//! before anything.
//!
//! The arithmetic is the x87's, in plain Rust over `X` (`math/x87_float.rs`); the tests compare
//! it with recorded vectors (`data/vectors`).

use crate::d3d::vertices_store::Vertex;
use crate::errors::{BadInput, Refused};
use crate::hud::owner::Hud;
use crate::hud::store::HudBlock;
use crate::math::x87_float::{ld, stf};

pub use rc2k_core::hud::disc::ENTRIES;

fn place(h: &HudBlock, k: u32, vertex: &Vertex) {
    let [u, v, x, y] = x87_1(h, k as usize);
    vertex.u.set(u);
    vertex.v.set(v);
    vertex.x.set(x);
    vertex.y.set(y);
}

/// Ring entry `entry` into `vertex`.
pub fn disc_vertex(hud: &Hud, entry: u32, vertex: &Vertex) -> Result<(), Refused> {
    if entry >= ENTRIES {
        return Err(BadInput("a disc entry past the ring").into());
    }
    {
        place(&hud.block, entry * 4, vertex);
        let colour = (hud.block.quad.alpha.get() << 24) | 0xFF_FFFF;
        vertex.colour.set(colour);
        vertex.specular.set(0);
    }
    Ok(())
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[e + 0x18] = (((([b + k + 0x8D0] * [b + 0x710]) * [b + 0x70C]) + [b + 0x6DC]) + [b + 0x714]) / [b + 0x6E0]`
/// `[e + 0x1C] = ((((-[b + k + 0x914] * [b + 0x710]) * [b + 0x70C]) + [b + 0x6DC]) + [b + 0x718]) / [b + 0x6E0]`
/// `[e + 0] = ([b + k + 0x848] * [b + 0x710]) + [b + 0x704]`
/// `[e + 4] = (-[b + k + 0x88C] * [b + 0x710]) + [b + 0x708]`
/// The four as stored, in their order: `[e + 0x18]`, `[e + 0x1C]`, `[e + 0]`, `[e + 4]`.
fn x87_1(h: &HudBlock, k: usize) -> [f32; 4] {
    let t0 = ld(h.texture_ring_sines[k / 4].get());
    let t1 = ld(h.disc_size.get());
    let t2 = t0 * t1;
    let t3 = ld(h.disc_texture_scale.get());
    let t4 = t2 * t3;
    let t5 = ld(h.disc_place_offset.get());
    let t6 = t4 + t5;
    let t7 = ld(h.disc_texture_shift[0].get());
    let t8 = t6 + t7;
    let t9 = ld(h.disc_place_divisor.get());
    let t10 = t8 / t9;
    let t10 = stf(t10);
    let t11 = ld(h.texture_ring_cosines[k / 4].get());
    let t12 = -t11;
    let t13 = ld(h.disc_size.get());
    let t14 = t12 * t13;
    let t15 = ld(h.disc_texture_scale.get());
    let t16 = t14 * t15;
    let t17 = ld(h.disc_place_offset.get());
    let t18 = t16 + t17;
    let t19 = ld(h.disc_texture_shift[1].get());
    let t20 = t18 + t19;
    let t21 = ld(h.disc_place_divisor.get());
    let t22 = t20 / t21;
    let t22 = stf(t22);
    let t23 = ld(h.ring_sines[k / 4].get());
    let t24 = ld(h.disc_size.get());
    let t25 = t23 * t24;
    let t26 = ld(h.disc_centre[0].get());
    let t27 = t25 + t26;
    let t27 = stf(t27);
    let t28 = ld(h.ring_cosines[k / 4].get());
    let t29 = -t28;
    let t30 = ld(h.disc_size.get());
    let t31 = t29 * t30;
    let t32 = ld(h.disc_centre[1].get());
    let t33 = t31 + t32;
    let t33 = stf(t33);
    [t10, t22, t27, t33]
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::vectors::{Block, In, Operand, Out, sparse};

    // The recorded run, one function per run.

    /// The four stored into the test's vertex at their offsets.
    fn put(stored: [f32; 4], vertex: &[std::cell::Cell<u32>]) {
        for (offset, value) in [0x18, 0x1C, 0, 4].into_iter().zip(stored) {
            vertex[offset / 4].set(value.to_bits());
        }
    }

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "disc_vertex line 25 idx k=0",
                operands: vec![
                    Operand::Sized {
                        len: size_of::<HudBlock>(),
                        reads: &[
                            (0x6DC, In::F),
                            (0x6E0, In::F),
                            (0x704, In::F),
                            (0x708, In::F),
                            (0x70C, In::F),
                            (0x710, In::F),
                            (0x714, In::F),
                            (0x718, In::F),
                            (0x848, In::F),
                            (0x88C, In::F),
                            (0x8D0, In::F),
                            (0x914, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::Value(0),
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::F), (4, Out::F), (0x18, Out::F), (0x1C, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| put(x87_1(a[0].view::<HudBlock>(), a[1].value()), a[2].cells()),
            },
            Block {
                name: "disc_vertex line 25 idx k=4",
                operands: vec![
                    Operand::Sized {
                        len: size_of::<HudBlock>(),
                        reads: &[
                            (0x6DC, In::F),
                            (0x6E0, In::F),
                            (0x704, In::F),
                            (0x708, In::F),
                            (0x70C, In::F),
                            (0x710, In::F),
                            (0x714, In::F),
                            (0x718, In::F),
                            (0x84C, In::F),
                            (0x890, In::F),
                            (0x8D4, In::F),
                            (0x918, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::Value(4),
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::F), (4, Out::F), (0x18, Out::F), (0x1C, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| put(x87_1(a[0].view::<HudBlock>(), a[1].value()), a[2].cells()),
            },
            Block {
                name: "disc_vertex line 25 idx k=36",
                operands: vec![
                    Operand::Sized {
                        len: size_of::<HudBlock>(),
                        reads: &[
                            (0x6DC, In::F),
                            (0x6E0, In::F),
                            (0x704, In::F),
                            (0x708, In::F),
                            (0x70C, In::F),
                            (0x710, In::F),
                            (0x714, In::F),
                            (0x718, In::F),
                            (0x86C, In::F),
                            (0x8B0, In::F),
                            (0x8F4, In::F),
                            (0x938, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::Value(36),
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::F), (4, Out::F), (0x18, Out::F), (0x1C, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| put(x87_1(a[0].view::<HudBlock>(), a[1].value()), a[2].cells()),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("hud.disc_vertex", &blocks(), 1200, 100);
    }
}
