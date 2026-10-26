//! `sound::speed_sound` (0x40DE09): a player's car's sound by its speed (+0x108, from
//! `sound::car_engine_state`), at the car.
//!
//! In:  ebx the car's sound object, edi its sound kind, esi the request
//! Out: eax the object's +0 (+ set +0 when stopped; +0x34 when placed), ebp the set
//!      (kind +0x60); everything else preserved
//! Nothing for a computer's car (+0 negative). The sound: kind +0x60's +0 + the
//! object's +0, into the request. With the state (+0x10C) 0 or 1, +0x24 1 (no gear),
//! +0xF4 or +0xAC not 0: `sound::stop_sound`. Else the request's +4 = +0x108 x set
//! +0x18 + set +0x10, +8 = +0x108 x set +0x24 + set +0x1C (x `flt_51637C` when +0xC is
//! 0), the place = +0x2C .. +0x34, and `sound::play_placed_sound`. Both are tail jumps.
//! One caller, `sound::car_sounds`.
//! Refuses (before anything) a null object or request, and a kind wanted but null.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::math::x87_float::ld;
use crate::pointers::GamePtr;
use crate::scene::x87_mem::{fld, fstp};
use rc2k_core::sound::engine::{computers, speed_stopped};

// ---- Rust version ----

/// A player's sound by the speed for object `o` of sound kind `kind`, through `request`.
pub fn speed_sound(o: u32, kind: u32, request: u32) -> Result<(), Refused> {
    if o == 0 || request == 0 {
        return Err(BadInput("no object or no request").into());
    }
    let so = crate::sound::store::SoundObject::at(o)?;
    // Converted: `rc2k_core::sound::engine` (`computers`, `speed_stopped`).
    if computers(so.sound_offset.get()) {
        return Ok(());
    }
    if kind == 0 {
        return Err(BadInput("no sound kind").into());
    }
    let set = kind + 0x60;
    let sound = so
        .sound_offset
        .get()
        .wrapping_add(GamePtr::new(set).read::<u32>("a sound set")?);
    crate::sound::turn_parts::put(request, 0, sound)?;
    if speed_stopped(
        so.state.get(),
        || so.no_gear.get(),
        || so.countdown.get(),
        || so.flag_ac.get(),
    ) {
        return crate::sound::turn_parts::stop(request);
    }
    x87_1(so, set as usize, request as usize, 0x18, 0x10, 4);
    if so.view_flag.get() == 0 {
        x87_2(
            so,
            set as usize,
            &crate::sound::store::SURFACE_SOUNDS,
            request as usize,
        );
    } else {
        x87_1(so, set as usize, request as usize, 0x24, 0x1C, 8);
    }
    for k in 0..3 {
        crate::sound::turn_parts::put(request, 3 + k, so.place[k].bits())?;
    }
    crate::sound::turn_parts::placed(request)
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[si + o3] = ([bx + 0x108] * [bp + o1]) + [bp + o2]`
fn x87_1(
    bx: &crate::sound::store::SoundObject,
    bp: usize,
    si: usize,
    o1: usize,
    o2: usize,
    o3: usize,
) {
    unsafe {
        let bp = bp as *const u8;
        let si = si as *mut u8;
        let t0 = ld(bx.speed_input.bits());
        let t1 = fld(bp, o1);
        let t2 = t0 * t1;
        let t3 = fld(bp, o2);
        let t4 = t2 + t3;
        fstp(si, o3, t4);
    }
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[si + 8] = (([bx + 0x108] * [bp + 0x24]) + [bp + 0x1C]) * [sf + 0x24]`
fn x87_2(
    bx: &crate::sound::store::SoundObject,
    bp: usize,
    sf: &crate::sound::store::SurfaceSounds,
    si: usize,
) {
    unsafe {
        let bp = bp as *const u8;
        let si = si as *mut u8;
        let t0 = ld(bx.speed_input.bits());
        let t1 = fld(bp, 0x24);
        let t2 = t0 * t1;
        let t3 = fld(bp, 0x1C);
        let t4 = t2 + t3;
        let t5 = ld(sf.speed_gain.bits());
        let t6 = t4 * t5;
        fstp(si, 8, t6);
    }
}

#[cfg(test)]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(bx, bp, si) "fld dword ptr [{bx} + 0x108]", "fmul dword ptr [{bp} + 0x18]", "fadd dword ptr [{bp} + 0x10]", "fstp dword ptr [{si} + 0x4]");
    reference!(ref_1(bx, bp, sf, si) "fld dword ptr [{bx} + 0x108]", "fmul dword ptr [{bp} + 0x24]", "fadd dword ptr [{bp} + 0x1C]", "fmul dword ptr [{sf} + 0x24]", "fstp dword ptr [{si} + 0x8]");
    reference!(ref_2(bx, bp, si) "fld dword ptr [{bx} + 0x108]", "fmul dword ptr [{bp} + 0x24]", "fadd dword ptr [{bp} + 0x1C]", "fstp dword ptr [{si} + 0x8]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "speed_sound line 44",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x108, In::F)],
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
                        unsafe { &*(a[0] as *const crate::sound::store::SoundObject) },
                        a[1],
                        a[2],
                        0x18,
                        0x10,
                        4,
                    )
                },
            },
            Block {
                name: "speed_sound line 52",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x108, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x1C, In::F), (0x24, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0x24, In::F)],
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
                        unsafe { &*(a[0] as *const crate::sound::store::SoundObject) },
                        a[1],
                        unsafe { &*(a[2] as *const crate::sound::store::SurfaceSounds) },
                        a[3],
                    )
                },
            },
            Block {
                name: "speed_sound line 61",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x108, In::F)],
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
                        unsafe { &*(a[0] as *const crate::sound::store::SoundObject) },
                        a[1],
                        a[2],
                        0x24,
                        0x1C,
                        8,
                    )
                },
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
