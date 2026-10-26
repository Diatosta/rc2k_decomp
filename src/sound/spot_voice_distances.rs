//! `sound::spot_voice_distances` (0x40E733): each voice heard this turn: its distance,
//! the distance's change and the pitch and volume factors, as `sound::car_engine_state`
//! does for a car.
//!
//! In:  nothing
//! Out: every register preserved (`pusha` / `popa`)
//! For each voice heard (+4 1): +0x40 the root of +8 .. +0x10 squared, +0x44 its change
//! x the turns a second (`dword_5124EC`); +0x4C 0 and +0x48 1.0, then unless the voice
//! is new (+0x20 1) +0x48 eased from 1.0 toward (+0x44 + 337.52) x 0.00296; +0x4C 1.0,
//! or 1 / the distance when that is over 1 (compared as integers); +0x20 cleared. See
//! `sound/spot_voices.rs`. One caller, `sound::spot_sounds`.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::math::x87_float::X;
use crate::math::x87_float::ld;
use crate::scene::x87_mem::{fld, fstp};
use crate::sound::spot_voices::{VOICE_COUNT, set, voice, vw};

// ---- Rust version ----

pub fn spot_voice_distances() -> Result<(), crate::errors::BadInput> {
    for n in 0..VOICE_COUNT {
        let bx = voice(n);
        if vw(bx, 4)? != 1 {
            continue;
        }
        x87_1(bx as usize, &crate::sound::store::RACE_CLOCK);
        set(bx, 0x4C, 0)?;
        set(bx, 0x48, 0x3F80_0000)?;
        if vw(bx, 0x20)? != 1 {
            x87_2(
                bx as usize,
                &crate::sound::store::RACE_CLOCK,
                &crate::sound::store::SURFACE_SOUNDS,
            );
        }
        set(bx, 0x4C, 0x3F80_0000)?;
        if vw(bx, 0x40)? as i32 > 0x3F80_0000 {
            x87_3(bx as usize);
        }
        set(bx, 0x20, 0)?;
    }
    Ok(())
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[bx + 0x40] = sqrt((([bx + 8] * [bx + 8]) + (([bx + 0xC] * [bx + 0xC]) + ([bx + 0x10] * [bx + 0x10]))))`
/// `[bx + 0x44] = ([bx + 0x40] - sqrt((([bx + 8] * [bx + 8]) + (([bx + 0xC] * [bx + 0xC]) + ([bx + 0x10] * [bx + 0x10]))))) * turns_a_second`
fn x87_1(bx: usize, ck: &crate::sound::store::RaceClock) {
    let t0 = fld(bx, 0x40);
    let t1 = fld(bx, 8);
    let t2 = fld(bx, 8);
    let t3 = t1 * t2;
    let t4 = fld(bx, 0xC);
    let t5 = fld(bx, 0xC);
    let t6 = t4 * t5;
    let t7 = fld(bx, 0x10);
    let t8 = fld(bx, 0x10);
    let t9 = t7 * t8;
    let t10 = t6 + t9;
    let t11 = t3 + t10;
    let t12 = t11.sqrt();
    fstp(bx, 0x40, t12);
    let t13 = t0 - t12;
    let t14 = ld(ck.turns_a_second.bits());
    let t15 = t13 * t14;
    fstp(bx, 0x44, t15);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[bx + 0x48] = (((([bx + 0x44] + sound_speed) * over_sound_speed) - [bx + 0x48]) * [sf + 8]) + [bx + 0x48]`
fn x87_2(bx: usize, ck: &crate::sound::store::RaceClock, sf: &crate::sound::store::SurfaceSounds) {
    let t0 = fld(bx, 0x44);
    let t1 = ld(ck.sound_speed.bits());
    let t2 = t0 + t1;
    let t3 = ld(ck.over_sound_speed.bits());
    let t4 = t2 * t3;
    let t5 = fld(bx, 0x48);
    let t6 = t4 - t5;
    let t7 = ld(sf.pitch_blend.bits());
    let t8 = t6 * t7;
    let t9 = fld(bx, 0x48);
    let t10 = t8 + t9;
    fstp(bx, 0x48, t10);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[bx + 0x4C] = 1 / [bx + 0x40]`
fn x87_3(bx: usize) {
    let t0 = X(1.0);
    let t1 = fld(bx, 0x40);
    let t2 = t0 / t1;
    fstp(bx, 0x4C, t2);
}

#[cfg(test)]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(bx, ck) "fld dword ptr [{bx} + 0x40]", "fld dword ptr [{bx} + 0x8]", "fmul dword ptr [{bx} + 0x8]", "fld dword ptr [{bx} + 0xC]", "fmul dword ptr [{bx} + 0xC]", "fld dword ptr [{bx} + 0x10]", "fmul dword ptr [{bx} + 0x10]", ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", ".byte 0xD9, 0xFA", "fst dword ptr [{bx} + 0x40]", ".byte 0xDE, 0xE9", "fmul dword ptr [{ck} + 8]", "fstp dword ptr [{bx} + 0x44]");
    reference!(ref_1(bx, ck, sf) "fld dword ptr [{bx} + 0x44]", "fadd dword ptr [{ck} + 0x14]", "fmul dword ptr [{ck} + 0x18]", "fsub dword ptr [{bx} + 0x48]", "fmul dword ptr [{sf} + 0x8]", "fadd dword ptr [{bx} + 0x48]", "fstp dword ptr [{bx} + 0x48]");
    reference!(ref_2(bx) ".byte 0xD9, 0xE8", "fdiv dword ptr [{bx} + 0x40]", "fstp dword ptr [{bx} + 0x4C]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "spot_voice_distances line 27",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(8, In::F), (0xC, In::F), (0x10, In::F), (0x40, In::F)],
                        writes: &[(0x40, Out::F), (0x44, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(8, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0], a[1]) },
                plain: |a| {
                    x87_1(a[0], unsafe {
                        &*(crate::pointers::host_at(a[1]) as *const crate::sound::store::RaceClock)
                    })
                },
            },
            Block {
                name: "spot_voice_distances line 47",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x44, In::F), (0x48, In::F)],
                        writes: &[(0x48, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0x14, In::F), (0x18, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(8, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0], a[1], a[2]) },
                plain: |a| {
                    x87_2(
                        a[0],
                        unsafe {
                            &*(crate::pointers::host_at(a[1])
                                as *const crate::sound::store::RaceClock)
                        },
                        unsafe {
                            &*(crate::pointers::host_at(a[2])
                                as *const crate::sound::store::SurfaceSounds)
                        },
                    )
                },
            },
            Block {
                name: "spot_voice_distances line 60",
                operands: vec![Operand::Buffer {
                    reads: &[(0x40, In::F)],
                    writes: &[(0x4C, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_2(a[0]) },
                plain: |a| x87_3(a[0]),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
