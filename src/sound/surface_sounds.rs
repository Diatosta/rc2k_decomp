//! `sound::surface_sounds` (0x40DE9B): a car's two sounds of what its wheels run on.
//!
//! Takes: the car's sound object, the request
//! The four wheels are records of five dwords at +0x44, +0x58, +0x6C, +0x80 (+8 and +0xC
//! floats, +0x10 the surface). From the floats at +8: `SurfaceSounds.across` (front less back,
//! by the signs), `SurfaceSounds.along` (left less right), +0x284 = the sum of the four +0
//! floats x 0.4 and +0x28C = the sum at +8 x 64 (`fistp`), `SurfaceSounds.volume` = the products'
//! sum x 0.1: a reading. The wheel whose surface counts is the front or back pair by
//! the sign bit of the first (negative: +0x44 / +0x6C), then the second of the pair when
//! the other is negative; its surface goes to +0x294 and `dword_4F66F1`; on 7 and 8 the
//! volume (`SurfaceSounds.volume`) x 0.25. The surface's sound set is `off_51639C[2 x
//! StageSounds.surface_sets[0][surface]]`, and `sound::surface_sound` (with the set and
//! the other wheel of the pair). Then the same from the floats at +0xC (+0x290 = their sum x 64, the volume
//! their sum x 0.1, the surface into `dword_4F66F5`, x 0.1 on 7 and 8) with the sets
//! of `off_5163A0`. The x87 code is plain Rust over `X` (`math/x87_float.rs`; the original's instructions
//! are recorded in the tests' `reference_vectors`; `Machine.scratch` is the scratch dword for the constants). One caller, `sound::car_sounds`.
//! Refuses a null object, a surface past the 16 of `StageSounds.surface_sets[0]` and a set
//! past the seven pairs: what was written is put back (a sound already sent stays).

use crate::errors::{BadInput, Refused};
use crate::math::x87_float::X;
use crate::math::x87_float::{ld, stf, to_int};
use crate::scene::x87_mem::{Base, fld, mov};
use crate::sound::sets_store::SoundRequest;
use crate::sound::store::{SoundObject, WHEEL_SURFACES};
use crate::system::owner::System;
use rc2k_core::sound::surfaces;

/// The surface of the wheel picked (of object `so`), its set and `sound::surface_sound`
/// (the other wheel of the pair is not needed there).
fn one(
    o: &SoundObject,
    request: &SoundRequest,
    second: usize,
    sound: &crate::sound::owner::Sound,
    system: &System,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::sound::surfaces` (`wheel`, `quieter`, `set_pair`).
    let (first, _) = surfaces::wheel(
        sound.race.surface.get().across.bits(),
        sound.race.surface.get().along.bits(),
    );
    let surface = o.voices[first].surface.get();
    if second == 0 {
        o.surface.set(surface);
    }
    WHEEL_SURFACES[second].set(surface);
    if surfaces::quieter(surface) {
        if second == 0 {
            x87_1(sound.race.surface.get(), &system.session.machine.scratch);
        } else {
            x87_2(sound.race.surface.get(), &system.session.machine.scratch);
        }
    }
    let pair = surfaces::set_pair(surface, |n| {
        sound.race.stage.get().surface_sets[n as usize].get()
    })
    .map_err(BadInput::from)?;
    let set = sound
        .race
        .surface_sets
        .get()
        .set(second + 2 * pair as usize)?;
    crate::sound::surface_sound::surface_sound(o, set, request, sound, audio)
}

/// The wheels' surface sounds of object `so`, through `request`.
pub fn surface_sounds(
    so: &crate::sound::store::SoundObject,
    request: &SoundRequest,
    sound: &crate::sound::owner::Sound,
    system: &System,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    x87_3(so, sound.race.surface.get());
    one(so, request, 0, sound, system, audio)?;
    x87_4(so, sound.race.surface.get());
    one(so, request, 1, sound, system, audio)
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [t + 0] = 0x3E800000`
/// `[sf + 0x40] = [sf + 0x40] * [t + 0]`
fn x87_1(sf: &crate::sound::store::SurfaceSounds, t: impl Base) {
    let t0 = ld(sf.volume.get());
    mov(t, 0, 0x3E800000);
    let t1 = fld(t, 0);
    let t2 = t0 * t1;
    sf.volume.set(stf(t2));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [t + 0] = 0x3DCCCCCD`
/// `[sf + 0x40] = [sf + 0x40] * [t + 0]`
fn x87_2(sf: &crate::sound::store::SurfaceSounds, t: impl Base) {
    let t0 = ld(sf.volume.get());
    mov(t, 0, 0x3DCCCCCD);
    let t1 = fld(t, 0);
    let t2 = t0 * t1;
    sf.volume.set(stf(t2));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[sf + 0x34] = (((0 - [bx + 0x4C]) + [bx + 0x60]) - [bx + 0x74]) + [bx + 0x88]`
/// `[sf + 0x3C] = (((0 + [bx + 0x4C]) + [bx + 0x60]) - [bx + 0x74]) - [bx + 0x88]`
/// `int [bx + 0x284] = ((([bx + 0x44] + [bx + 0x58]) + [bx + 0x6C]) + [bx + 0x80]) * [sf + 0xC]`
/// `int [bx + 0x28C] = ((([bx + 0x4C] + [bx + 0x60]) + [bx + 0x74]) + [bx + 0x88]) * [sf + 0x10]`
/// (a long formula: checked by the tests' `reference_vectors`)
fn x87_3(bx: &crate::sound::store::SoundObject, sf: &crate::sound::store::SurfaceSounds) {
    let t0 = X(0.0);
    let t1 = ld(bx.voices[0].share_b.get());
    let t2 = t0 - t1;
    let t3 = ld(bx.voices[1].share_b.get());
    let t4 = t2 + t3;
    let t5 = ld(bx.voices[2].share_b.get());
    let t6 = t4 - t5;
    let t7 = ld(bx.voices[3].share_b.get());
    let t8 = t6 + t7;
    sf.across.set(stf(t8));
    let t9 = X(0.0);
    let t10 = ld(bx.voices[0].share_b.get());
    let t11 = t9 + t10;
    let t12 = ld(bx.voices[1].share_b.get());
    let t13 = t11 + t12;
    let t14 = ld(bx.voices[2].share_b.get());
    let t15 = t13 - t14;
    let t16 = ld(bx.voices[3].share_b.get());
    let t17 = t15 - t16;
    sf.along.set(stf(t17));
    let t18 = ld(bx.voices[0].share_a.get());
    let t19 = ld(bx.voices[1].share_a.get());
    let t20 = t18 + t19;
    let t21 = ld(bx.voices[2].share_a.get());
    let t22 = t20 + t21;
    let t23 = ld(bx.voices[3].share_a.get());
    let t24 = t22 + t23;
    let t25 = ld(sf.across_scale.get());
    let t26 = t24 * t25;
    bx.sum_a.set(to_int(t26) as u32);
    let t27 = ld(bx.voices[0].share_b.get());
    let t28 = ld(bx.voices[1].share_b.get());
    let t29 = t27 + t28;
    let t30 = ld(bx.voices[2].share_b.get());
    let t31 = t29 + t30;
    let t32 = ld(bx.voices[3].share_b.get());
    let t33 = t31 + t32;
    let t34 = ld(sf.along_scale.get());
    let t35 = t33 * t34;
    bx.sum_b.set(to_int(t35) as u32);
    let t36 = X(0.0);
    let t37 = ld(bx.voices[0].share_a.get());
    let t38 = ld(bx.voices[0].share_b.get());
    let t39 = t37 * t38;
    let t40 = t36 + t39;
    let t41 = ld(bx.voices[1].share_a.get());
    let t42 = ld(bx.voices[1].share_b.get());
    let t43 = t41 * t42;
    let t44 = t40 + t43;
    let t45 = ld(bx.voices[2].share_a.get());
    let t46 = ld(bx.voices[2].share_b.get());
    let t47 = t45 * t46;
    let t48 = t44 + t47;
    let t49 = ld(bx.voices[3].share_a.get());
    let t50 = ld(bx.voices[3].share_b.get());
    let t51 = t49 * t50;
    let t52 = t48 + t51;
    let t53 = ld(sf.volume_scale.get());
    let t54 = t52 * t53;
    sf.volume.set(stf(t54));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[sf + 0x34] = (((0 - [bx + 0x50]) + [bx + 0x64]) - [bx + 0x78]) + [bx + 0x8C]`
/// `[sf + 0x3C] = (((0 + [bx + 0x50]) + [bx + 0x64]) - [bx + 0x78]) - [bx + 0x8C]`
/// `int [bx + 0x290] = ((([bx + 0x50] + [bx + 0x64]) + [bx + 0x78]) + [bx + 0x8C]) * [sf + 0x10]`
/// `[sf + 0x40] = ((([bx + 0x50] + [bx + 0x64]) + [bx + 0x78]) + [bx + 0x8C]) * [sf + 4]`
fn x87_4(bx: &crate::sound::store::SoundObject, sf: &crate::sound::store::SurfaceSounds) {
    let t0 = X(0.0);
    let t1 = ld(bx.voices[0].share_c.get());
    let t2 = t0 - t1;
    let t3 = ld(bx.voices[1].share_c.get());
    let t4 = t2 + t3;
    let t5 = ld(bx.voices[2].share_c.get());
    let t6 = t4 - t5;
    let t7 = ld(bx.voices[3].share_c.get());
    let t8 = t6 + t7;
    sf.across.set(stf(t8));
    let t9 = X(0.0);
    let t10 = ld(bx.voices[0].share_c.get());
    let t11 = t9 + t10;
    let t12 = ld(bx.voices[1].share_c.get());
    let t13 = t11 + t12;
    let t14 = ld(bx.voices[2].share_c.get());
    let t15 = t13 - t14;
    let t16 = ld(bx.voices[3].share_c.get());
    let t17 = t15 - t16;
    sf.along.set(stf(t17));
    let t18 = ld(bx.voices[0].share_c.get());
    let t19 = ld(bx.voices[1].share_c.get());
    let t20 = t18 + t19;
    let t21 = ld(bx.voices[2].share_c.get());
    let t22 = t20 + t21;
    let t23 = ld(bx.voices[3].share_c.get());
    let t24 = t22 + t23;
    let t25 = ld(sf.along_scale.get());
    let t26 = t24 * t25;
    bx.sum_c.set(to_int(t26) as u32);
    let t27 = ld(bx.voices[0].share_c.get());
    let t28 = ld(bx.voices[1].share_c.get());
    let t29 = t27 + t28;
    let t30 = ld(bx.voices[2].share_c.get());
    let t31 = t29 + t30;
    let t32 = ld(bx.voices[3].share_c.get());
    let t33 = t31 + t32;
    let t34 = ld(sf.volume_scale.get());
    let t35 = t33 * t34;
    sf.volume.set(stf(t35));
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "surface_sounds line 62",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x40, In::F)],
                        writes: &[(0x40, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[(0, Out::I)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    x87_1(
                        unsafe {
                            &*(crate::test_support::vectors::block_vectors::at(a[0])
                                as *const crate::sound::store::SurfaceSounds)
                        },
                        a[1],
                    );
                },
            },
            Block {
                name: "surface_sounds line 70",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x40, In::F)],
                        writes: &[(0x40, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[(0, Out::I)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    x87_2(
                        unsafe {
                            &*(crate::test_support::vectors::block_vectors::at(a[0])
                                as *const crate::sound::store::SurfaceSounds)
                        },
                        a[1],
                    );
                },
            },
            Block {
                name: "surface_sounds line 92",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (0x44, In::F),
                            (0x4C, In::F),
                            (0x58, In::F),
                            (0x60, In::F),
                            (0x6C, In::F),
                            (0x74, In::F),
                            (0x80, In::F),
                            (0x88, In::F),
                        ],
                        writes: &[(0x284, Out::I), (0x28C, Out::I)],
                    },
                    Operand::Buffer {
                        reads: &[(4, In::F), (0xC, In::F), (0x10, In::F)],
                        writes: &[(0x34, Out::F), (0x3C, Out::F), (0x40, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    x87_3(
                        unsafe {
                            &*(crate::test_support::vectors::block_vectors::at(a[0])
                                as *const crate::sound::store::SoundObject)
                        },
                        unsafe {
                            &*(crate::test_support::vectors::block_vectors::at(a[1])
                                as *const crate::sound::store::SurfaceSounds)
                        },
                    );
                },
            },
            Block {
                name: "surface_sounds line 135",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x50, In::F), (0x64, In::F), (0x78, In::F), (0x8C, In::F)],
                        writes: &[(0x290, Out::I)],
                    },
                    Operand::Buffer {
                        reads: &[(4, In::F), (0x10, In::F)],
                        writes: &[(0x34, Out::F), (0x3C, Out::F), (0x40, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    x87_4(
                        unsafe {
                            &*(crate::test_support::vectors::block_vectors::at(a[0])
                                as *const crate::sound::store::SoundObject)
                        },
                        unsafe {
                            &*(crate::test_support::vectors::block_vectors::at(a[1])
                                as *const crate::sound::store::SurfaceSounds)
                        },
                    );
                },
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("sound.surface_sounds", &blocks(), 1200, 100);
    }
}
