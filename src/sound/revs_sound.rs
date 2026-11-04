//! `sound::revs_sound` (0x40DD19): a car's engine sound by its revs, placed ahead of the
//! car.
//!
//! In:  ebx the car's sound object, edi its sound kind, esi the request
//! Out: eax the sound, ebp its set (kind +0x10, or `unk_516230` for a computer's car);
//!      everything else preserved
//! The sound: the set's +0 + the object's +0 (a computer's car, +0 negative: + its
//! bit 0, from `unk_516230`), into the request. With the state (+0x10C) 0 or 3:
//! `sound::stop_sound`. Else the request's +4 (pitch) = +0x100 x set +0x18 + set +0x10,
//! +8 (volume) = +0x100 x set +0x24 + set +0x1C (x `flt_516380` when +0xC is 0), the
//! place = +0x2C .. +0x34 plus +0x38 .. +0x40, and `sound::play_placed_sound`. Both
//! are tail jumps. One caller, `sound::car_sounds`.
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
use rc2k_core::sound::engine::{computers, revs_sound_number, revs_stopped};

// ---- Rust version ----

/// The engine's sound for object `so` of sound kind number `kind`, through `request`.
pub fn revs_sound(
    so: &crate::sound::store::SoundObject,
    kind: u32,
    request: &SoundRequest,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::sound::engine` (`computers`, `revs_sound_number`,
    // `revs_stopped`).
    let object = so.sound_offset.get();
    let set = if computers(object) {
        crate::sound::sets_store::SoundKinds::computers_set()
    } else {
        crate::sound::sets_store::SoundKinds::record(kind)?.offset(0x10)
    };
    let sound = revs_sound_number(object, set.read::<u32>("a sound set")?);
    request.sound.set(sound);
    if revs_stopped(so.state.get()) {
        return crate::sound::turn_parts::stop(request);
    }
    x87_1(so, set, &request.pitch, 0x18, 0x10);
    if so.view_flag.get() == 0 {
        x87_2(
            so,
            set,
            &crate::sound::store::SURFACE_SOUNDS,
            &request.volume,
        );
    } else {
        x87_1(so, set, &request.volume, 0x24, 0x1C);
    }
    x87_3(so, &request.place);
    crate::sound::turn_parts::placed(request)
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `out = ([bx + 0x100] * [bp + o1]) + [bp + o2]` (`out` the request's +4 or +8)
fn x87_1(
    bx: &crate::sound::store::SoundObject,
    bp: GamePtr,
    out: &impl Cell32,
    o1: usize,
    o2: usize,
) {
    let t0 = ld(bx.revs_input.bits());
    let t1 = fld(bp, o1);
    let t2 = t0 * t1;
    let t3 = fld(bp, o2);
    let t4 = t2 + t3;
    out.set_bits(st(t4));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `volume = (([bx + 0x100] * [bp + 0x24]) + [bp + 0x1C]) * [sf + 0x28]` (the request's +8)
fn x87_2(
    bx: &crate::sound::store::SoundObject,
    bp: GamePtr,
    sf: &crate::sound::store::SurfaceSounds,
    volume: &impl Cell32,
) {
    let t0 = ld(bx.revs_input.bits());
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
/// `place[0] = [bx + 0x2C] + [bx + 0x38]`
/// `place[1] = [bx + 0x30] + [bx + 0x3C]`
/// `place[2] = [bx + 0x34] + [bx + 0x40]` (the request's +0xC .. +0x14)
fn x87_3(bx: &crate::sound::store::SoundObject, place: &[impl Cell32; 3]) {
    let t0 = ld(bx.place[0].bits());
    let t1 = ld(bx.motion[0].bits());
    let t2 = t0 + t1;
    place[0].set_bits(st(t2));
    let t3 = ld(bx.place[1].bits());
    let t4 = ld(bx.motion[1].bits());
    let t5 = t3 + t4;
    place[1].set_bits(st(t5));
    let t6 = ld(bx.place[2].bits());
    let t7 = ld(bx.motion[2].bits());
    let t8 = t6 + t7;
    place[2].set_bits(st(t8));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};
    use crate::scene::x87_mem::HostCell;

    // The original's instructions, one function per run.
    reference!(ref_0(bx, bp, si) "fld dword ptr [{bx} + 0x100]", "fmul dword ptr [{bp} + 0x18]", "fadd dword ptr [{bp} + 0x10]", "fstp dword ptr [{si} + 0x4]");
    reference!(ref_1(bx, bp, sf, si) "fld dword ptr [{bx} + 0x100]", "fmul dword ptr [{bp} + 0x24]", "fadd dword ptr [{bp} + 0x1C]", "fmul dword ptr [{sf} + 0x28]", "fstp dword ptr [{si} + 0x8]");
    reference!(ref_2(bx, bp, si) "fld dword ptr [{bx} + 0x100]", "fmul dword ptr [{bp} + 0x24]", "fadd dword ptr [{bp} + 0x1C]", "fstp dword ptr [{si} + 0x8]");
    reference!(ref_3(bx, si) "fld dword ptr [{bx} + 0x2C]", "fadd dword ptr [{bx} + 0x38]", "fstp dword ptr [{si} + 0xC]", "fld dword ptr [{bx} + 0x30]", "fadd dword ptr [{bx} + 0x3C]", "fstp dword ptr [{si} + 0x10]", "fld dword ptr [{bx} + 0x34]", "fadd dword ptr [{bx} + 0x40]", "fstp dword ptr [{si} + 0x14]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "revs_sound line 52",
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
                        unsafe {
                            &*(crate::pointers::host_at(a[0])
                                as *const crate::sound::store::SoundObject)
                        },
                        crate::pointers::GamePtr::new(a[1] as u32),
                        &HostCell(a[2] + 4),
                        0x18,
                        0x10,
                    );
                },
            },
            Block {
                name: "revs_sound line 60",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x100, In::F)],
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
                name: "revs_sound line 69",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x100, In::F)],
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
                        unsafe {
                            &*(crate::pointers::host_at(a[0])
                                as *const crate::sound::store::SoundObject)
                        },
                        crate::pointers::GamePtr::new(a[1] as u32),
                        &HostCell(a[2] + 8),
                        0x24,
                        0x1C,
                    );
                },
            },
            Block {
                name: "revs_sound line 77",
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
