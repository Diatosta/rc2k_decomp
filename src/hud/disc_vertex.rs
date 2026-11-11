//! `hud::disc_vertex` (0x4B6D10): a vertex of the HUD's disc from its ring entry: the texture
//! place; the place; colour white under the quad record's alpha, specular 0.
//!
//! Takes: the ring entry (0 the centre .. 0x10), the vertex
//! Returns: the colour
//!
//! Caller: `hud::draw_disc` (three a triangle). Refuses an entry past 0x10, before anything.
//!
//! The arithmetic is the x87's, in plain Rust over `X` (`math/x87_float.rs`); the original's
//! instructions are `asm_reference` in the tests ..

use crate::d3d::vertices_store::Vertex;
use crate::errors::{BadInput, Refused};
use crate::hud::owner::Hud;
use crate::hud::store::HudBlock;
use crate::math::x87_float::ld;
use crate::scene::x87_mem::{Base, fstp};

pub use rc2k_core::hud::disc::ENTRIES;

fn place(h: &HudBlock, k: u32, vertex: &Vertex) {
    x87_1(h, k as usize, vertex);
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

// ---- the arithmetic that was x87 assembly ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[e + 0x18] = (((([b + k + 0x8D0] * [b + 0x710]) * [b + 0x70C]) + [b + 0x6DC]) + [b + 0x714]) / [b + 0x6E0]`
/// `[e + 0x1C] = ((((-[b + k + 0x914] * [b + 0x710]) * [b + 0x70C]) + [b + 0x6DC]) + [b + 0x718]) / [b + 0x6E0]`
/// `[e + 0] = ([b + k + 0x848] * [b + 0x710]) + [b + 0x704]`
/// `[e + 4] = (-[b + k + 0x88C] * [b + 0x710]) + [b + 0x708]`
fn x87_1(h: &HudBlock, k: usize, e: impl Base) {
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
    fstp(e, 0x18, t10);
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
    fstp(e, 0x1C, t22);
    let t23 = ld(h.ring_sines[k / 4].get());
    let t24 = ld(h.disc_size.get());
    let t25 = t23 * t24;
    let t26 = ld(h.disc_centre[0].get());
    let t27 = t25 + t26;
    fstp(e, 0, t27);
    let t28 = ld(h.ring_cosines[k / 4].get());
    let t29 = -t28;
    let t30 = ld(h.disc_size.get());
    let t31 = t29 * t30;
    let t32 = ld(h.disc_centre[1].get());
    let t33 = t31 + t32;
    fstp(e, 4, t33);
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(b, k, e) "fld dword ptr [{b} + {k} + 0x8D0]", "fmul dword ptr [{b} + 0x710]", "fmul dword ptr [{b} + 0x70C]", "fadd dword ptr [{b} + 0x6DC]", "fadd dword ptr [{b} + 0x714]", "fdiv dword ptr [{b} + 0x6E0]", "fstp dword ptr [{e} + 0x18]", "fld dword ptr [{b} + {k} + 0x914]", "fchs", "fmul dword ptr [{b} + 0x710]", "fmul dword ptr [{b} + 0x70C]", "fadd dword ptr [{b} + 0x6DC]", "fadd dword ptr [{b} + 0x718]", "fdiv dword ptr [{b} + 0x6E0]", "fstp dword ptr [{e} + 0x1C]", "fld dword ptr [{b} + {k} + 0x848]", "fmul dword ptr [{b} + 0x710]", "fadd dword ptr [{b} + 0x704]", "fstp dword ptr [{e}]", "fld dword ptr [{b} + {k} + 0x88C]", "fchs", "fmul dword ptr [{b} + 0x710]", "fadd dword ptr [{b} + 0x708]", "fstp dword ptr [{e} + 4]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "disc_vertex line 25 idx k=0",
                operands: vec![
                    Operand::Buffer {
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
                reference: |a| unsafe { ref_0(a[0], a[1], a[2]) },
                plain: |a| {
                    x87_1(
                        unsafe { &*(crate::pointers::host_at(a[0]) as *const HudBlock) },
                        a[1],
                        a[2],
                    );
                },
            },
            Block {
                name: "disc_vertex line 25 idx k=4",
                operands: vec![
                    Operand::Buffer {
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
                reference: |a| unsafe { ref_0(a[0], a[1], a[2]) },
                plain: |a| {
                    x87_1(
                        unsafe { &*(crate::pointers::host_at(a[0]) as *const HudBlock) },
                        a[1],
                        a[2],
                    );
                },
            },
            Block {
                name: "disc_vertex line 25 idx k=36",
                operands: vec![
                    Operand::Buffer {
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
                reference: |a| unsafe { ref_0(a[0], a[1], a[2]) },
                plain: |a| {
                    x87_1(
                        unsafe { &*(crate::pointers::host_at(a[0]) as *const HudBlock) },
                        a[1],
                        a[2],
                    );
                },
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
