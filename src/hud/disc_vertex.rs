//! A vertex of the HUD's disc from its ring entry (0 the centre .. 0x10): the texture place; the
//! place; colour white under the quad record's alpha, specular 0. Refuses an entry past 0x10,
//! before anything.
//!
//! The arithmetic is the x87's, in plain Rust over `X` (`math/x87_float.rs`); the tests compare
//! it with recorded vectors (`data/vectors`).

use crate::d3d::vertices_store::Vertex;
use crate::errors::{BadInput, Refused};
use crate::hud::block_store::HudBlock;
use crate::hud::owner::Hud;
use crate::math::x87_float::{ld, stf};

pub use rc2k_core::hud::disc::ENTRIES;

/// The variables one vertex's arithmetic reads: the disc's constants and the ring's entry.
struct Entry {
    place_offset: f32,
    place_divisor: f32,
    centre: [f32; 2],
    texture_scale: f32,
    size: f32,
    texture_shift: [f32; 2],
    sine: f32,
    cosine: f32,
    texture_sine: f32,
    texture_cosine: f32,
}

impl Entry {
    /// Ring entry `at` of the block's disc.
    fn of(h: &HudBlock, at: usize) -> Self {
        Self {
            place_offset: h.disc_place_offset,
            place_divisor: h.disc_place_divisor,
            centre: h.disc_centre,
            texture_scale: h.disc_texture_scale,
            size: h.disc_size,
            texture_shift: h.disc_texture_shift,
            sine: h.ring_sines[at],
            cosine: h.ring_cosines[at],
            texture_sine: h.texture_ring_sines[at],
            texture_cosine: h.texture_ring_cosines[at],
        }
    }
}

fn place(h: &HudBlock, entry: u32, vertex: &Vertex) {
    let [u, v, x, y] = x87_1(&Entry::of(h, entry as usize));
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
        place(&hud.block, entry, vertex);
        let colour = (hud.block.quad.alpha << 24) | 0xFF_FFFF;
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
fn x87_1(h: &Entry) -> [f32; 4] {
    let t0 = ld(h.texture_sine);
    let t1 = ld(h.size);
    let t2 = t0 * t1;
    let t3 = ld(h.texture_scale);
    let t4 = t2 * t3;
    let t5 = ld(h.place_offset);
    let t6 = t4 + t5;
    let t7 = ld(h.texture_shift[0]);
    let t8 = t6 + t7;
    let t9 = ld(h.place_divisor);
    let t10 = t8 / t9;
    let t10 = stf(t10);
    let t11 = ld(h.texture_cosine);
    let t12 = -t11;
    let t13 = ld(h.size);
    let t14 = t12 * t13;
    let t15 = ld(h.texture_scale);
    let t16 = t14 * t15;
    let t17 = ld(h.place_offset);
    let t18 = t16 + t17;
    let t19 = ld(h.texture_shift[1]);
    let t20 = t18 + t19;
    let t21 = ld(h.place_divisor);
    let t22 = t20 / t21;
    let t22 = stf(t22);
    let t23 = ld(h.sine);
    let t24 = ld(h.size);
    let t25 = t23 * t24;
    let t26 = ld(h.centre[0]);
    let t27 = t25 + t26;
    let t27 = stf(t27);
    let t28 = ld(h.cosine);
    let t29 = -t28;
    let t30 = ld(h.size);
    let t31 = t29 * t30;
    let t32 = ld(h.centre[1]);
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

    /// The entry the recorded run read: the block's dwords at the recorded byte positions (the
    /// game's layout of the block), the ring entry `k / 4`.
    fn entry_of(block: &[std::cell::Cell<u32>], k: usize) -> Entry {
        let float = |offset: usize| f32::from_bits(block[offset / 4].get());
        let ring = |start: usize| float(start + k);
        Entry {
            place_offset: float(0x6DC),
            place_divisor: float(0x6E0),
            centre: [float(0x704), float(0x708)],
            texture_scale: float(0x70C),
            size: float(0x710),
            texture_shift: [float(0x714), float(0x718)],
            sine: ring(0x848),
            cosine: ring(0x88C),
            texture_sine: ring(0x8D0),
            texture_cosine: ring(0x914),
        }
    }

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "disc_vertex line 25 idx k=0",
                operands: vec![
                    Operand::Sized {
                        len: 0x1000,
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
                plain: |a| put(x87_1(&entry_of(a[0].cells(), a[1].value())), a[2].cells()),
            },
            Block {
                name: "disc_vertex line 25 idx k=4",
                operands: vec![
                    Operand::Sized {
                        len: 0x1000,
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
                plain: |a| put(x87_1(&entry_of(a[0].cells(), a[1].value())), a[2].cells()),
            },
            Block {
                name: "disc_vertex line 25 idx k=36",
                operands: vec![
                    Operand::Sized {
                        len: 0x1000,
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
                plain: |a| put(x87_1(&entry_of(a[0].cells(), a[1].value())), a[2].cells()),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("hud.disc_vertex", &blocks(), 1200, 100);
    }
}
