//! `sound::throttle_sound` (0x40DD96): a player's car's engine sound by its throttle,
//! placed behind the car.
//!
//! In:  ebx the car's sound object, edi its sound kind, esi the request
//! Out: eax the object's +0 (+ set +0 when sent), ebp the set (kind +0x38) when sent;
//!      everything else preserved
//! Nothing for a computer's car (+0 negative). The sound: kind +0x38's +0 + the
//! object's +0, into the request. With the state (+0x10C) 0 or 3: `sound::stop_sound`.
//! Else the request's +4 = +0x100 x set +0x18 + set +0x10, +8 = +0x104 x set +0x24 +
//! set +0x1C (x `flt_516380` when +0xC is 0), the place = +0x2C .. +0x34 less +0x38 ..
//! +0x40, and `sound::play_placed_sound`. Both are tail jumps. One caller,
//! `sound::car_sounds`.
//! Refuses (before anything) a null object, and a kind wanted but null.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::math::x87_float::{ld, st};
use crate::pointers::GamePtr;
use crate::scene::x87_mem::fld;
use crate::session::Cell32;
use crate::sound::sets_store::SoundRequest;
use rc2k_core::sound::engine::{computers, revs_stopped};

// ---- Rust version ----

/// A player's engine sound by the throttle for object `so` of sound kind number `kind`,
/// through `request`.
pub fn throttle_sound(
    so: &crate::sound::store::SoundObject,
    kind: u32,
    request: &SoundRequest,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::sound::engine` (`computers`, `revs_stopped`).
    if computers(so.sound_offset.get()) {
        return Ok(());
    }
    let set = crate::sound::sets_store::SoundKinds::record(kind)?.offset(0x38);
    let sound = so
        .sound_offset
        .get()
        .wrapping_add(set.read::<u32>("a sound set")?);
    request.sound.set(sound);
    if revs_stopped(so.state.get()) {
        return crate::sound::turn_parts::stop(request);
    }
    x87_1(&so.revs_input, set, &request.pitch, 0x18, 0x10);
    if so.view_flag.get() == 0 {
        x87_2(
            so,
            set,
            &crate::sound::store::SURFACE_SOUNDS,
            &request.volume,
        );
    } else {
        x87_1(&so.throttle_input, set, &request.volume, 0x24, 0x1C);
    }
    x87_3(so, &request.place);
    crate::sound::turn_parts::placed(request)
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `out = (input * [bp + o1]) + [bp + o2]` (`out` the request's +4 or +8)
fn x87_1(input: &crate::session::Float, bp: GamePtr, out: &impl Cell32, o1: usize, o2: usize) {
    let t0 = ld(input.bits());
    let t1 = fld(bp, o1);
    let t2 = t0 * t1;
    let t3 = fld(bp, o2);
    let t4 = t2 + t3;
    out.set_bits(st(t4));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `volume = (([bx + 0x104] * [bp + 0x24]) + [bp + 0x1C]) * [sf + 0x28]` (the request's +8)
fn x87_2(
    bx: &crate::sound::store::SoundObject,
    bp: GamePtr,
    sf: &crate::sound::store::SurfaceSounds,
    volume: &impl Cell32,
) {
    let t0 = ld(bx.throttle_input.bits());
    let t1 = fld(bp, 0x24);
    let t2 = t0 * t1;
    let t3 = fld(bp, 0x1C);
    let t4 = t2 + t3;
    let t5 = ld(sf.revs_gain.bits());
    let t6 = t4 * t5;
    volume.set_bits(st(t6));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `place[0] = [bx + 0x2C] - [bx + 0x38]`
/// `place[1] = [bx + 0x30] - [bx + 0x3C]`
/// `place[2] = [bx + 0x34] - [bx + 0x40]` (the request's +0xC .. +0x14)
fn x87_3(bx: &crate::sound::store::SoundObject, place: &[impl Cell32; 3]) {
    let t0 = ld(bx.place[0].bits());
    let t1 = ld(bx.motion[0].bits());
    let t2 = t0 - t1;
    place[0].set_bits(st(t2));
    let t3 = ld(bx.place[1].bits());
    let t4 = ld(bx.motion[1].bits());
    let t5 = t3 - t4;
    place[1].set_bits(st(t5));
    let t6 = ld(bx.place[2].bits());
    let t7 = ld(bx.motion[2].bits());
    let t8 = t6 - t7;
    place[2].set_bits(st(t8));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};
    use crate::scene::x87_mem::HostCell;

    // The original's instructions, one function per run.
    reference!(ref_0(bx, bp, si) "fld dword ptr [{bx} + 0x100]", "fmul dword ptr [{bp} + 0x18]", "fadd dword ptr [{bp} + 0x10]", "fstp dword ptr [{si} + 0x4]");
    reference!(ref_1(bx, bp, sf, si) "fld dword ptr [{bx} + 0x104]", "fmul dword ptr [{bp} + 0x24]", "fadd dword ptr [{bp} + 0x1C]", "fmul dword ptr [{sf} + 0x28]", "fstp dword ptr [{si} + 0x8]");
    reference!(ref_2(bx, bp, si) "fld dword ptr [{bx} + 0x104]", "fmul dword ptr [{bp} + 0x24]", "fadd dword ptr [{bp} + 0x1C]", "fstp dword ptr [{si} + 0x8]");
    reference!(ref_3(bx, si) "fld dword ptr [{bx} + 0x2C]", "fsub dword ptr [{bx} + 0x38]", "fstp dword ptr [{si} + 0xC]", "fld dword ptr [{bx} + 0x30]", "fsub dword ptr [{bx} + 0x3C]", "fstp dword ptr [{si} + 0x10]", "fld dword ptr [{bx} + 0x34]", "fsub dword ptr [{bx} + 0x40]", "fstp dword ptr [{si} + 0x14]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "throttle_sound line 45",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x100, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x10, In::F), (0x18, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(4, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0], a[1], a[2]) },
                plain: |a| {
                    x87_1(
                        &unsafe {
                            &*(crate::pointers::host_at(a[0])
                                as *const crate::sound::store::SoundObject)
                        }
                        .revs_input,
                        crate::pointers::GamePtr::new(a[1] as u32),
                        &HostCell(a[2] + 4),
                        0x18,
                        0x10,
                    );
                },
            },
            Block {
                name: "throttle_sound line 53",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x104, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x1C, In::F), (0x24, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x28, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(8, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0], a[1], a[2], a[3]) },
                plain: |a| {
                    x87_2(
                        unsafe {
                            &*(crate::pointers::host_at(a[0])
                                as *const crate::sound::store::SoundObject)
                        },
                        crate::pointers::GamePtr::new(a[1] as u32),
                        unsafe {
                            &*(crate::pointers::host_at(a[2])
                                as *const crate::sound::store::SurfaceSounds)
                        },
                        &HostCell(a[3] + 8),
                    );
                },
            },
            Block {
                name: "throttle_sound line 62",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x104, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x1C, In::F), (0x24, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(8, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_2(a[0], a[1], a[2]) },
                plain: |a| {
                    x87_1(
                        &unsafe {
                            &*(crate::pointers::host_at(a[0])
                                as *const crate::sound::store::SoundObject)
                        }
                        .throttle_input,
                        crate::pointers::GamePtr::new(a[1] as u32),
                        &HostCell(a[2] + 8),
                        0x24,
                        0x1C,
                    );
                },
            },
            Block {
                name: "throttle_sound line 70",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (0x2C, In::F),
                            (0x30, In::F),
                            (0x34, In::F),
                            (0x38, In::F),
                            (0x3C, In::F),
                            (0x40, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0xC, Out::F), (0x10, Out::F), (0x14, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_3(a[0], a[1]) },
                plain: |a| {
                    x87_3(
                        unsafe {
                            &*(crate::pointers::host_at(a[0])
                                as *const crate::sound::store::SoundObject)
                        },
                        &[
                            HostCell(a[1] + 0xC),
                            HostCell(a[1] + 0x10),
                            HostCell(a[1] + 0x14),
                        ],
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
