//! `hud::disc_vertex` (0x4B6D10): vertex edi of the HUD's disc from its ring entry ecx:
//! the texture place (`flt_75C210`[ecx], -`flt_75C254`[ecx], times `dword_75C050` times
//! `dword_75C04C`, plus `flt_75C01C` and `dword_75C054` / `dword_75C058`, over
//! `flt_75C020`); the place (`flt_75C188`[ecx], -`flt_75C1CC`[ecx], times `dword_75C050`,
//! plus `dword_75C044` / `dword_75C048`); colour white under the quad record's alpha,
//! specular 0.
//!
//! In:  ecx the ring entry (0 the centre .. 0x10), edi the vertex
//! Out: eax the colour; the rest kept
//!
//! Caller: `hud::draw_disc` (three a triangle). Refuses an entry past 0x10, before anything.
//!
//! The arithmetic is the x87's, in plain Rust over `X` (`math/x87_float.rs`); the original's
//! instructions are `asm_reference` in the tests (2026-10-24).

use crate::errors::{BadInput, Refused};
use crate::hud::add_quad::QUAD_RVA;
use crate::hud::store::game_read;
use crate::scene::x87_mem::{fld, fstp};

pub use rc2k_core::hud::disc::ENTRIES;

unsafe fn place(b: u32, k: u32, e: u32) {
    x87_1(b as usize, k as usize, e as usize)
}

/// The Rust version (see the header): ring entry `entry` into the vertex at `vertex`.
pub unsafe fn disc_vertex(entry: u32, vertex: u32) -> Result<(), Refused> {
    if entry >= ENTRIES {
        return Err(BadInput("a disc entry past the ring").into());
    }
    unsafe {
        place(crate::hud::store::base(), entry * 4, vertex);
        let colour = (game_read::<u32>(QUAD_RVA) << 24) | 0xFF_FFFF;
        ((vertex + 0x10) as *mut u32).write_unaligned(colour);
        ((vertex + 0x14) as *mut u32).write_unaligned(0);
    }
    Ok(())
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[e + 0x18] = (((([b + k + 0x8D0] * [b + 0x710]) * [b + 0x70C]) + [b + 0x6DC]) + [b + 0x714]) / [b + 0x6E0]`
/// `[e + 0x1C] = ((((-[b + k + 0x914] * [b + 0x710]) * [b + 0x70C]) + [b + 0x6DC]) + [b + 0x718]) / [b + 0x6E0]`
/// `[e + 0] = ([b + k + 0x848] * [b + 0x710]) + [b + 0x704]`
/// `[e + 4] = (-[b + k + 0x88C] * [b + 0x710]) + [b + 0x708]`
fn x87_1(b: usize, k: usize, e: usize) {
    unsafe {
        let b = b as *const u8;
        let e = e as *mut u8;
        let t0 = fld(b, k + 0x8D0);
        let t1 = fld(b, 0x710);
        let t2 = t0 * t1;
        let t3 = fld(b, 0x70C);
        let t4 = t2 * t3;
        let t5 = fld(b, 0x6DC);
        let t6 = t4 + t5;
        let t7 = fld(b, 0x714);
        let t8 = t6 + t7;
        let t9 = fld(b, 0x6E0);
        let t10 = t8 / t9;
        fstp(e, 0x18, t10);
        let t11 = fld(b, k + 0x914);
        let t12 = -t11;
        let t13 = fld(b, 0x710);
        let t14 = t12 * t13;
        let t15 = fld(b, 0x70C);
        let t16 = t14 * t15;
        let t17 = fld(b, 0x6DC);
        let t18 = t16 + t17;
        let t19 = fld(b, 0x718);
        let t20 = t18 + t19;
        let t21 = fld(b, 0x6E0);
        let t22 = t20 / t21;
        fstp(e, 0x1C, t22);
        let t23 = fld(b, k + 0x848);
        let t24 = fld(b, 0x710);
        let t25 = t23 * t24;
        let t26 = fld(b, 0x704);
        let t27 = t25 + t26;
        fstp(e, 0, t27);
        let t28 = fld(b, k + 0x88C);
        let t29 = -t28;
        let t30 = fld(b, 0x710);
        let t31 = t29 * t30;
        let t32 = fld(b, 0x708);
        let t33 = t31 + t32;
        fstp(e, 4, t33);
    }
}

#[cfg(test)]
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
                plain: |a| x87_1(a[0], a[1], a[2]),
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
                plain: |a| x87_1(a[0], a[1], a[2]),
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
                plain: |a| x87_1(a[0], a[1], a[2]),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
