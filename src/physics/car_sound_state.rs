//! `physics::car_sound_state` (0x45CACE): the work car's engine sound object
//! (`g_raceSoundObjects[g_currentCar & 7]`) told the car's state: +0x10 1; +0x14 the revs
//! (+0x1E4); +0x54, +0x68, +0x7C, +0x90 `g_car` +0x930; the share +0x28C =
//! +0x288 / `flt_613EE0` and +0x290 1.0 (with +0x254 set, those and +0x294 0), into the
//! object's four voices (+0x4C .., x256 at +0x50 .., +0x294 at +0x44 ..); +0x18 +0x638
//! held to 0 ..= 0x100; +0xE8 the gear; +0x1C +0x1C0; +0xAC +0x248 & +0x24C; +0x0C
//! +0x6F0 ^ 1; the car's place less the listener's (`g_viewBlocks[0]`'s with
//! `dword_63DB94` 1, else the work car's own +0x6A4 ..) turned by `g_viewTurnMatrix`, x
//! 1/65536 into +0x2C ..; the car's +0xD4 .. turned into +0x38 ... All in `pusha` /
//! `popa`.
//!
//! the exact x87 one (the runs over the object, plain Rust over `X`) stays
//! for `physics::engine_sound`.
//!
//! In:  nothing
//! Out: every register kept; no flags read
//!
//! Caller: `physics::engine_sound`. Refuses (before anything) a null sound object, or a
//! null view block when it is read.

use crate::errors::{BadInput, Refused};
use crate::math::x87_float::{X, st};
use crate::math::x87_run::{ext_times, ints_diff_80};
use crate::physics::car_store::CAR;
use crate::race::matrices_store::MATRICES;
use crate::race::numbers_store::NUMBERS;
use crate::race::view_store::RACE_VIEW;
use crate::race::work_store::RACE_WORK;
use crate::scene::matrix3::{extended, rounded};
use crate::scene::rotate_point::rotate_point;
use crate::scene::x87_mem::{fld, fstp};
use crate::session::Dword;

fn get(a: u32) -> u32 {
    crate::pointers::read_or_end::<u32>(a, "physics::car_sound_state's operand")
}

/// 0x45CB02 .. 0x45CB88 (the shares into the voices), the integer steps between in Rust.
fn shares(so: &crate::sound::store::SoundObject) {
    x87_1(crate::race::store::asm_base() as usize);
    if CAR.all_wheels_off.get() != 0 {
        CAR.sound_slip_share.set_bits(0);
        CAR.sound_level.set_bits(0);
        CAR.sound_speed.set_bits(0);
    }
    x87_2(crate::race::store::asm_base() as usize, so);
}

fn object() -> u32 {
    crate::sound::store::RACE_SOUNDS.table[(RACE_WORK.current_car.get() & 7) as usize].get()
}

fn listener() -> u32 {
    if RACE_VIEW.players.get() == 1 {
        RACE_VIEW.view_blocks[0].get()
    } else {
        crate::physics::car_store::car_addr()
    }
}

/// The exact x87 version (for `physics::engine_sound`).
pub fn car_sound_state() -> Result<(), Refused> {
    let o = object();
    if o == 0 {
        return Err(BadInput("a null sound object").into());
    }
    let l = listener();
    if l == 0 {
        return Err(BadInput("a null view block").into());
    }
    let so = crate::sound::store::SoundObject::at(o)?;
    so.new_revs.set(1);
    so.revs.set(CAR.engine_sound_revs.get());
    let w = CAR.wheel_surface_a[4].get();
    for voice in &so.voices {
        voice.surface.set(w);
    }
    shares(so);
    let n = CAR.throttle.get() as i32;
    so.throttle.set(n.clamp(0, 0x100) as u32);
    so.gear.set(CAR.gear.get());
    so.speed.set_bits(CAR.mean_drive_speed.bits());
    so.flag_ac
        .set(CAR.front_wheels_off.get() & CAR.rear_wheels_off.get());
    so.view_flag.set(CAR.camera_mode_is_2_or_3.get() ^ 1);
    let d = |c: &Dword, k: u32| ints_diff_80(c.get() as i32, get(l + k) as i32);
    let place = [
        d(&CAR.position[0], 0x6A4),
        d(&CAR.position[1], 0x6A8),
        d(&CAR.position[2], 0x6AC),
    ];
    let m = MATRICES.view_turn_matrix[0].addr();
    let out = rotate_point(m, place);
    let k = NUMBERS.one_over_65536.bits();
    for (i, e) in out.iter().enumerate() {
        so.place[i].set_bits(rounded(ext_times(*e, k)));
    }
    let out = rotate_point(
        m,
        [
            extended(CAR.crash_matrix[8].bits()),
            extended(CAR.crash_matrix[9].bits()),
            extended(CAR.crash_matrix[10].bits()),
        ],
    );
    for (i, e) in out.iter().enumerate() {
        so.motion[i].set_bits(rounded(*e));
    }
    Ok(())
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x31A264] = [rc + 0x31A260] / [rc + 0x213EE0]`
/// `[rc + 0x31A268] = 1`
fn x87_1(rc: usize) {
    let t0 = fld(rc, 0x31A260);
    let t1 = fld(rc, 0x213EE0);
    let t2 = t0 / t1;
    fstp(rc, 0x31A264, t2);
    let t3 = X(1.0);
    fstp(rc, 0x31A268, t3);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[o + 0x4C] = [rc + 0x31A268]`
/// `[o + 0x60] = [rc + 0x31A268]`
/// `[o + 0x74] = [rc + 0x31A268]`
/// `[o + 0x88] = [rc + 0x31A268]`
/// `[o + 0x50] = [rc + 0x31A264] * [rc + 0x1C5E2C]`
/// `[o + 0x64] = [rc + 0x31A264] * [rc + 0x1C5E2C]`
/// `[o + 0x78] = [rc + 0x31A264] * [rc + 0x1C5E2C]`
/// `[o + 0x8C] = [rc + 0x31A264] * [rc + 0x1C5E2C]`
/// `[o + 0x44] = [rc + 0x31A26C]`
/// `[o + 0x58] = [rc + 0x31A26C]`
/// `[o + 0x6C] = [rc + 0x31A26C]`
/// `[o + 0x80] = [rc + 0x31A26C]`
fn x87_2(rc: usize, o: &crate::sound::store::SoundObject) {
    let t0 = fld(rc, 0x31A268);
    o.voices[0].share_b.set_bits(st(t0));
    o.voices[1].share_b.set_bits(st(t0));
    o.voices[2].share_b.set_bits(st(t0));
    o.voices[3].share_b.set_bits(st(t0));
    let t1 = fld(rc, 0x31A264);
    let t2 = fld(rc, 0x1C5E2C);
    let t3 = t1 * t2;
    o.voices[0].share_c.set_bits(st(t3));
    o.voices[1].share_c.set_bits(st(t3));
    o.voices[2].share_c.set_bits(st(t3));
    o.voices[3].share_c.set_bits(st(t3));
    let t4 = fld(rc, 0x31A26C);
    o.voices[0].share_a.set_bits(st(t4));
    o.voices[1].share_a.set_bits(st(t4));
    o.voices[2].share_a.set_bits(st(t4));
    o.voices[3].share_a.set_bits(st(t4));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(rc) "fld dword ptr [{rc} + 0x31A260]", "fdiv dword ptr [{rc} + 0x213EE0]", "fstp dword ptr [{rc} + 0x31A264]", ".byte 0xD9, 0xE8", "fstp dword ptr [{rc} + 0x31A268]");
    reference!(ref_1(rc, o) "fld dword ptr [{rc} + 0x31A268]", "fst dword ptr [{o} + 0x4C]", "fst dword ptr [{o} + 0x60]", "fst dword ptr [{o} + 0x74]", "fstp dword ptr [{o} + 0x88]", "fld dword ptr [{rc} + 0x31A264]", "fmul dword ptr [{rc} + 0x1C5E2C]", "fst dword ptr [{o} + 0x50]", "fst dword ptr [{o} + 0x64]", "fst dword ptr [{o} + 0x78]", "fstp dword ptr [{o} + 0x8C]", "fld dword ptr [{rc} + 0x31A26C]", "fst dword ptr [{o} + 0x44]", "fst dword ptr [{o} + 0x58]", "fst dword ptr [{o} + 0x6C]", "fstp dword ptr [{o} + 0x80]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "car_sound_state line 46",
                operands: vec![Operand::Buffer {
                    reads: &[(0x213EE0, In::F), (0x31A260, In::F)],
                    writes: &[(0x31A264, Out::F), (0x31A268, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0]) },
                plain: |a| x87_1(a[0]),
            },
            Block {
                name: "car_sound_state line 64",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (0x1C5E2C, In::F),
                            (0x31A264, In::F),
                            (0x31A268, In::F),
                            (0x31A26C, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[
                            (0x44, Out::F),
                            (0x4C, Out::F),
                            (0x50, Out::F),
                            (0x58, Out::F),
                            (0x60, Out::F),
                            (0x64, Out::F),
                            (0x6C, Out::F),
                            (0x74, Out::F),
                            (0x78, Out::F),
                            (0x80, Out::F),
                            (0x88, Out::F),
                            (0x8C, Out::F),
                        ],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0], a[1]) },
                plain: |a| {
                    x87_2(a[0], unsafe {
                        &*(crate::pointers::host_at(a[1])
                            as *const crate::sound::store::SoundObject)
                    })
                },
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
