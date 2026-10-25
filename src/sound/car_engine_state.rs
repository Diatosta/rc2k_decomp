//! `sound::car_engine_state` (0x40D8E2): a car's sound object brought up to date for the
//! turn: the revs eased between the physics' readings, the pitch and volume inputs of
//! the engine sounds, the engine's state, the gear-change and back-fire cues, the
//! listener's distance and the Doppler-like factors.
//!
//! In:  ebx the object
//! Out: edi the object's sound kind (`off_514F90[+4]`, 0x88 bytes each), eax its +0x134
//!      (the new distance), ecx +0x18 (or the back-fire's wait), edx as it came or the
//!      remainder of the revs' `idiv`; ebx, esi, ebp preserved
//! The object's dwords by offset (a reading of the code, names by use):
//! - +0x10 1: a new revs reading (+0x14) came: +0x94/+0x98 the last two readings' times
//!   (`dword_5124E8`), +0x9C/+0xA0 their revs, +0x10 cleared. Else, unless
//!   `dword_5BEC94` is 1, +0x14 = the revs extrapolated (`imul` / `idiv` over the
//!   readings' gap held to 1000 ms), at least 1000.
//! - +0x100 = (+0x14 - kind +4) x kind +0xC; +0x104 = +0x18 as a float over 256 (the
//!   exponent less 8 unless it is 0); +0x108 = |+0x1C| x `flt_516370`.
//! - +0x110 = the last state, +0x10C = 0 (no revs), 1 (up to 1000), 2, or 3 when +0x20
//!   was set (cleared).
//! - +0x118 follows +0x18 (up: by (0x100 - it) >> 6 and one more, down: by it >> 7 and
//!   one more, never past), +0x11C the gap, +0x12C 0 / 1 up / 2 down, +0x128 / +0x130
//!   the turns going up / down (at 1 made 2 when level).
//! - +0xFC the last direction; up then down (or the reverse: the two add to 3): +0xF4
//!   += 5. +0xE8 (the gear) changed: +0xEC = it, +0xF4 += 7, and with kind bit 0x2000, a
//!   gear over 1 and +0x11C at least 0x32: +0x1FC and +0xF0 1, +0x210 = (clock & 0xF) x
//!   1/60 + 0.75 (through `dword_5183CC`). +0x24 = (eax is 0): the gear 0, or (in that
//!   last case) the clock's low four bits 0.
//! - Kind bit 0x1000 (back-fires): going up clears +0x1E0, +0x1E4, +0x1F4, +0x1EC; else
//!   with +0x118 at most 0xF0, +0x1F4 counts down and below 0 is set to (clock & 3) +
//!   (max(+0x14 - 2000, 0) >> 9) + 1, +0x1E0 1 and +0x1EC += 8 held to 0x20. Then going
//!   up clears +0xF8; else unless +0xF8 is 1, going down by more than 20: with +0x14 at
//!   least 4000 +0x1C4 = kind +0 & 0xFFF and +0x1C8 1 (the slot of
//!   `sound::play_timed_slot`), and +0xF8 1.
//! - +0xF4 counts down to 0. +0x134 the distance (the root of +0x2C .. +0x34 squared),
//!   +0x138 its change x the turns a second, +0x13C eased toward (+0x138 + 337.52) x
//!   0.00296, +0x140 = 1, or 1 / the distance when that is over 1 (compared as integers).
//!
//! One caller, `sound::car_sounds`.
//! Refuses (before writing) a kind past the 36 of the table and a revs extrapolation
//! whose quotient does not fit (the original's `idiv` traps).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::math::x87::from_int;
use crate::math::x87_float::{X, int, ld, st};
use crate::scene::x87_mem::{fild, fld, mov};
use crate::session::MACHINE;
use rc2k_core::sound::engine::{Left, Words, extrapolation, over_256, readings, state};

/// off_514F90: the sound kinds, 36 pointers.
pub const KINDS: usize = 0x514F90;
pub const KIND_COUNT: u32 = 36;
/// dword_5BEC94: 1 holds the revs as they are.
const HOLD: usize = 0x5BEC94;

fn g(x: impl crate::store::Ida) -> u32 {
    unsafe { crate::store::ida_read(x) }
}

/// The kind of object `o` (its +4), or a refusal.
pub fn kind_of(o: u32) -> Result<u32, BadInput> {
    let n = unsafe { ((o + 4) as *const u32).read_unaligned() };
    if n >= KIND_COUNT {
        return Err(BadInput("a sound kind past the 36 of the table"));
    }
    Ok(g(KINDS + 4 * n as usize))
}

// ---- Rust version ----

/// The object's dwords where the original reads and writes them.
struct Object(u32);

impl Words for Object {
    fn get(&self, at: u32) -> u32 {
        unsafe { ((self.0 + at) as *const u32).read_unaligned() }
    }
    fn set(&mut self, at: u32, value: u32) {
        unsafe { ((self.0 + at) as *mut u32).write_unaligned(value) }
    }
}

/// The object `o` brought up to date; its sound kind returned (the original's edi).
pub fn car_engine_state(o: u32) -> Result<u32, Refused> {
    // Converted: `rc2k_core::sound::engine` (`extrapolation`, `readings`, `state`,
    // `over_256`); the floats are the original's x87 still.
    if o == 0 {
        return Err(BadInput("a null sound object").into());
    }
    let kind = kind_of(o)?;
    let mut object = Object(o);
    let clock = crate::sound::store::RACE_CLOCK.last.get();
    let (bx, di) = (o, kind);
    // The revs extrapolated, worked out first: its `idiv` may trap.
    let revs = extrapolation(&object, clock, || g(HOLD)).map_err(BadInput::from)?;
    // The registers the core models (written before they are read).
    let mut left = Left::default();
    readings(&mut object, clock, revs, || g(HOLD), &mut left);
    let so = unsafe { crate::sound::store::SoundObject::at(bx) };
    x87_1(so, di as usize);
    object.set(0x104, over_256(from_int(object.get(0x18) as i32)));
    x87_2(so, &crate::sound::store::SURFACE_SOUNDS);
    let flags = || unsafe { (kind as *const u32).read_unaligned() };
    if let Some(low) = state(&mut object, flags, clock, &mut left) {
        MACHINE.scratch.set(low);
        x87_3(crate::session::MACHINE.scratch.addr(), so);
    }
    x87_4(so, &crate::sound::store::RACE_CLOCK);
    object.set(0x140, 0);
    x87_5(
        so,
        &crate::sound::store::RACE_CLOCK,
        &crate::sound::store::SURFACE_SOUNDS,
    );
    object.set(0x140, 0x3F80_0000);
    if object.get(0x134) as i32 > 0x3F80_0000 {
        x87_6(so);
    }
    Ok(kind)
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[bx + 0x100] = (int [bx + 0x14] - [di + 4]) * [di + 0xC]`
fn x87_1(bx: &crate::sound::store::SoundObject, di: usize) {
    unsafe {
        let di = di as *const u8;
        let t0 = int(bx.revs.get() as i32);
        let t1 = fld(di, 4);
        let t2 = t0 - t1;
        let t3 = fld(di, 0xC);
        let t4 = t2 * t3;
        bx.revs_input.set_bits(st(t4));
    }
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[bx + 0x108] = |[bx + 0x1C]| * [sf + 0x18]`
fn x87_2(bx: &crate::sound::store::SoundObject, sf: &crate::sound::store::SurfaceSounds) {
    let t0 = ld(bx.speed.bits());
    let t1 = t0.abs();
    let t2 = ld(sf.speed_scale.bits());
    let t3 = t1 * t2;
    bx.speed_input.set_bits(st(t3));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [t + 0] = 0x3C888889`
/// `int [t + 0] = 0x3F400000`
/// `[bx + 0x210] = (int [t + 0] * [t + 0]) + [t + 0]`
fn x87_3(t: usize, bx: &crate::sound::store::SoundObject) {
    unsafe {
        let t = t as *mut u8;
        let t0 = fild(t, 0);
        mov(t, 0, 0x3C888889);
        let t1 = fld(t, 0);
        let t2 = t0 * t1;
        mov(t, 0, 0x3F400000);
        let t3 = fld(t, 0);
        let t4 = t2 + t3;
        bx.gear_pitch.set_bits(st(t4));
    }
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[bx + 0x134] = sqrt((([bx + 0x2C] * [bx + 0x2C]) + (([bx + 0x30] * [bx + 0x30]) + ([bx + 0x34] * [bx + 0x34]))))`
/// `[bx + 0x138] = ([bx + 0x134] - sqrt((([bx + 0x2C] * [bx + 0x2C]) + (([bx + 0x30] * [bx + 0x30]) + ([bx + 0x34] * [bx + 0x34]))))) * turns_a_second`
fn x87_4(bx: &crate::sound::store::SoundObject, ck: &crate::sound::store::RaceClock) {
    let t0 = ld(bx.distance.bits());
    let t1 = ld(bx.place[0].bits());
    let t2 = ld(bx.place[0].bits());
    let t3 = t1 * t2;
    let t4 = ld(bx.place[1].bits());
    let t5 = ld(bx.place[1].bits());
    let t6 = t4 * t5;
    let t7 = ld(bx.place[2].bits());
    let t8 = ld(bx.place[2].bits());
    let t9 = t7 * t8;
    let t10 = t6 + t9;
    let t11 = t3 + t10;
    let t12 = t11.sqrt();
    bx.distance.set_bits(st(t12));
    let t13 = t0 - t12;
    let t14 = ld(ck.turns_a_second.bits());
    let t15 = t13 * t14;
    bx.approach.set_bits(st(t15));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[bx + 0x13C] = (((([bx + 0x138] + sound_speed) * over_sound_speed) - [bx + 0x13C]) * [sf + 8]) + [bx + 0x13C]`
fn x87_5(
    bx: &crate::sound::store::SoundObject,
    ck: &crate::sound::store::RaceClock,
    sf: &crate::sound::store::SurfaceSounds,
) {
    let t0 = ld(bx.approach.bits());
    let t1 = ld(ck.sound_speed.bits());
    let t2 = t0 + t1;
    let t3 = ld(ck.over_sound_speed.bits());
    let t4 = t2 * t3;
    let t5 = ld(bx.pitch_factor.bits());
    let t6 = t4 - t5;
    let t7 = ld(sf.pitch_blend.bits());
    let t8 = t6 * t7;
    let t9 = ld(bx.pitch_factor.bits());
    let t10 = t8 + t9;
    bx.pitch_factor.set_bits(st(t10));
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[bx + 0x140] = 1 / [bx + 0x134]`
fn x87_6(bx: &crate::sound::store::SoundObject) {
    let t0 = X(1.0);
    let t1 = ld(bx.distance.bits());
    let t2 = t0 / t1;
    bx.volume_factor.set_bits(st(t2));
}

#[cfg(test)]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(bx, di) "fild dword ptr [{bx} + 0x14]", "fsub dword ptr [{di} + 0x4]", "fmul dword ptr [{di} + 0xC]", "fstp dword ptr [{bx} + 0x100]");
    reference!(ref_1(bx, sf) "fld dword ptr [{bx} + 0x1C]", ".byte 0xD9, 0xE1", "fmul dword ptr [{sf} + 0x18]", "fstp dword ptr [{bx} + 0x108]");
    reference!(ref_2(t, bx) "fild dword ptr [{t}]", "mov dword ptr [{t}], 0x3C888889", "fmul dword ptr [{t}]", "mov dword ptr [{t}], 0x3F400000", "fadd dword ptr [{t}]", "fstp dword ptr [{bx} + 0x210]");
    reference!(ref_3(bx, ck) "fld dword ptr [{bx} + 0x134]", "fld dword ptr [{bx} + 0x2C]", "fmul dword ptr [{bx} + 0x2C]", "fld dword ptr [{bx} + 0x30]", "fmul dword ptr [{bx} + 0x30]", "fld dword ptr [{bx} + 0x34]", "fmul dword ptr [{bx} + 0x34]", ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", ".byte 0xD9, 0xFA", "fst dword ptr [{bx} + 0x134]", ".byte 0xDE, 0xE9", "fmul dword ptr [{ck} + 8]", "fstp dword ptr [{bx} + 0x138]");
    reference!(ref_4(bx, ck, sf) "fld dword ptr [{bx} + 0x138]", "fadd dword ptr [{ck} + 0x14]", "fmul dword ptr [{ck} + 0x18]", "fsub dword ptr [{bx} + 0x13C]", "fmul dword ptr [{sf} + 0x8]", "fadd dword ptr [{bx} + 0x13C]", "fstp dword ptr [{bx} + 0x13C]");
    reference!(ref_5(bx) ".byte 0xD9, 0xE8", "fdiv dword ptr [{bx} + 0x134]", "fstp dword ptr [{bx} + 0x140]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "car_engine_state line 100",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x14, In::I)],
                        writes: &[(0x100, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(4, In::F), (0xC, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0], a[1]) },
                plain: |a| {
                    x87_1(
                        unsafe { &*(a[0] as *const crate::sound::store::SoundObject) },
                        a[1],
                    )
                },
            },
            Block {
                name: "car_engine_state line 108",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x1C, In::F)],
                        writes: &[(0x108, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0x18, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0], a[1]) },
                plain: |a| {
                    x87_2(
                        unsafe { &*(a[0] as *const crate::sound::store::SoundObject) },
                        unsafe { &*(a[1] as *const crate::sound::store::SurfaceSounds) },
                    )
                },
            },
            Block {
                name: "car_engine_state line 118",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[(0, Out::I)],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0x210, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_2(a[0], a[1]) },
                plain: |a| {
                    x87_3(a[0], unsafe {
                        &*(a[1] as *const crate::sound::store::SoundObject)
                    })
                },
            },
            Block {
                name: "car_engine_state line 128",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x2C, In::F), (0x30, In::F), (0x34, In::F), (0x134, In::F)],
                        writes: &[(0x134, Out::F), (0x138, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(8, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_3(a[0], a[1]) },
                plain: |a| {
                    x87_4(
                        unsafe { &*(a[0] as *const crate::sound::store::SoundObject) },
                        unsafe { &*(a[1] as *const crate::sound::store::RaceClock) },
                    )
                },
            },
            Block {
                name: "car_engine_state line 146",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x138, In::F), (0x13C, In::F)],
                        writes: &[(0x13C, Out::F)],
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
                reference: |a| unsafe { ref_4(a[0], a[1], a[2]) },
                plain: |a| {
                    x87_5(
                        unsafe { &*(a[0] as *const crate::sound::store::SoundObject) },
                        unsafe { &*(a[1] as *const crate::sound::store::RaceClock) },
                        unsafe { &*(a[2] as *const crate::sound::store::SurfaceSounds) },
                    )
                },
            },
            Block {
                name: "car_engine_state line 158",
                operands: vec![Operand::Buffer {
                    reads: &[(0x134, In::F)],
                    writes: &[(0x140, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_5(a[0]) },
                plain: |a| x87_6(unsafe { &*(a[0] as *const crate::sound::store::SoundObject) }),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
