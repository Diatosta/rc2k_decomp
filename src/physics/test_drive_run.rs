//! `physics::test_drive_run` (0x421848): the car's acceleration measured for its
//! figures: from standing (the drive speeds 0, gear 1, the engine 0, `unk_71A010` 1), 5000
//! steps of the engine and gearbox alone (in `pusha` / `popa`: the car's speed the drive
//! speed, `physics::count_down_timers`, `physics::gearbox_step`,
//! `physics::engine_torque`, the drive force `flt_614748` x `flt_5CB968` / the gear's ratio
//! / `flt_5C5E34` into `flt_614750`, `physics::drive_torque_share`, the drag (`fst` into
//! `dword_62400C`) and the drive force step the drive speeds, `physics::revs_from_wheels`);
//! the steps at which the speed first reaches `flt_5CB988` and `flt_5CB98C` into
//! `dword_5CB97C`, `dword_5CB980`; every twentieth step the speed x `flt_5C5D00` and the
//! gear into `flt_5CB990[k]`, `dword_5CBD90[k]`; at the end the speed x `flt_5C5D00` into
//! `dword_5CB984` and the two steps / `flt_5C5D14` (times) back into theirs.
//!
//! In:  nothing
//! Out: ebx 5000, eax the last step / 20, ecx 20, edx its remainder (the gear when 0); the
//!      rest kept; no flags read
//!
//! Caller: `physics::test_drive`. Refuses a gear outside -1 ..= 7 (past `g_gearRatios`) and
//! what its calls refuse; everything written put back.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::{BadInput, Refused};
use crate::physics::flow::{a, c0, g, p};
use crate::scene::x87_mem::{fild, fld, fstp};

const DRIVE: usize = 0x71A190; // g_carDriveSpeed
const GEAR: usize = 0x71A1AC; // g_carGear
pub const STEPS: u32 = 0x1388;

/// One step (0x42189D .., in `pusha` / `popa`).
fn step() -> Result<(), Refused> {
    p(0x71A184, crate::math::x87::loaded(g(DRIVE)));
    unsafe {
        crate::physics::count_down_timers::count_down_timers();
        crate::physics::gearbox_step::gearbox_step()?;
        crate::physics::engine_torque::engine_torque();
    }
    let gear = g(GEAR);
    if !(-1..=7).contains(&(gear as i32)) {
        return Err(BadInput("a gear outside g_gearRatios (-1 ..= 7)").into());
    }
    x87_1(
        crate::race::store::asm_base() as usize,
        (a(0x6140C0).wrapping_add(gear.wrapping_mul(4))) as usize,
    );
    unsafe { crate::physics::drive_torque_share::drive_torque_share() };
    x87_2(crate::race::store::asm_base() as usize);
    unsafe { crate::physics::revs_from_wheels::revs_from_wheels() }?;
    Ok(())
}

fn steps() -> Result<(), Refused> {
    for x in [0x5CB97C, 0x5CB980, 0x5CB984, DRIVE, 0x71A194, 0x71A1B0] {
        p(x, 0);
    }
    p(GEAR, 1);
    p(0x71A010, 1);
    for n in 0..STEPS {
        step()?;
        if g(0x5CB97C) == 0 && !c0(g(DRIVE), g(0x5CB988)) {
            p(0x5CB97C, n);
        }
        if g(0x5CB980) == 0 && !c0(g(DRIVE), g(0x5CB98C)) {
            p(0x5CB980, n);
        }
        if n % 20 == 0 {
            let slot = 4 * (n / 20) as usize;
            p(
                0x5CB990 + slot,
                crate::math::x87::mul(g(DRIVE), g(0x5C5D00)),
            );
            p(0x5CBD90 + slot, g(GEAR));
        }
    }
    x87_3(crate::race::store::asm_base() as usize);
    Ok(())
}

/// The Rust version.
pub fn test_drive_run() -> Result<(), Refused> {
    steps()
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x214750] = (([rc + 0x214748] * [rc + 0x1CB968]) / [r + 0]) / [rc + 0x1C5E34]`
fn x87_1(rc: usize, r: usize) {
    unsafe {
        let rc = rc as *mut u8;
        let r = r as *const u8;
        let t0 = fld(rc, 0x214748);
        let t1 = fld(rc, 0x1CB968);
        let t2 = t0 * t1;
        let t3 = fld(r, 0);
        let t4 = t2 / t3;
        let t5 = fld(rc, 0x1C5E34);
        let t6 = t4 / t5;
        fstp(rc, 0x214750, t6);
    }
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x22400C] = -(((([rc + 0x31A190] * [rc + 0x31A190]) * [rc + 0x213FCC]) / [rc + 0x1C5E34]) + [rc + 0x213FD0])`
/// (a long formula: see the instructions in the tests' `asm_reference`)
/// (a long formula: see the instructions in the tests' `asm_reference`)
fn x87_2(rc: usize) {
    unsafe {
        let rc = rc as *mut u8;
        let t0 = fld(rc, 0x31A190);
        let t1 = t0 * t0;
        let t2 = fld(rc, 0x213FCC);
        let t3 = t1 * t2;
        let t4 = fld(rc, 0x1C5E34);
        let t5 = t3 / t4;
        let t6 = fld(rc, 0x213FD0);
        let t7 = t5 + t6;
        let t8 = -t7;
        fstp(rc, 0x22400C, t8);
        let t9 = fld(rc, 0x214750);
        let t10 = t8 + t9;
        let t11 = fld(rc, 0x21388C);
        let t12 = t10 * t11;
        let t13 = fld(rc, 0x31A190);
        let t14 = t12 + t13;
        fstp(rc, 0x31A190, t14);
        fstp(rc, 0x31A194, t14);
    }
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x1CB984] = [rc + 0x31A190] * [rc + 0x1C5D00]`
/// `[rc + 0x1CB97C] = int [rc + 0x1CB97C] / [rc + 0x1C5D14]`
/// `[rc + 0x1CB980] = int [rc + 0x1CB980] / [rc + 0x1C5D14]`
fn x87_3(rc: usize) {
    unsafe {
        let rc = rc as *mut u8;
        let t0 = fld(rc, 0x31A190);
        let t1 = fld(rc, 0x1C5D00);
        let t2 = t0 * t1;
        fstp(rc, 0x1CB984, t2);
        let t3 = fild(rc, 0x1CB97C);
        let t4 = fld(rc, 0x1C5D14);
        let t5 = t3 / t4;
        fstp(rc, 0x1CB97C, t5);
        let t6 = fild(rc, 0x1CB980);
        let t7 = fld(rc, 0x1C5D14);
        let t8 = t6 / t7;
        fstp(rc, 0x1CB980, t8);
    }
}

#[cfg(test)]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(rc, r) "fld dword ptr [{rc} + 0x214748]", "fmul dword ptr [{rc} + 0x1CB968]", "fdiv dword ptr [{r}]", "fdiv dword ptr [{rc} + 0x1C5E34]", "fstp dword ptr [{rc} + 0x214750]");
    reference!(ref_1(rc) "fld dword ptr [{rc} + 0x31A190]", ".byte 0xD8, 0xC8", "fmul dword ptr [{rc} + 0x213FCC]", "fdiv dword ptr [{rc} + 0x1C5E34]", "fadd dword ptr [{rc} + 0x213FD0]", ".byte 0xD9, 0xE0", "fst dword ptr [{rc} + 0x22400C]", "fadd dword ptr [{rc} + 0x214750]", "fmul dword ptr [{rc} + 0x21388C]", "fadd dword ptr [{rc} + 0x31A190]", "fst dword ptr [{rc} + 0x31A190]", "fstp dword ptr [{rc} + 0x31A194]");
    reference!(ref_2(rc) "fld dword ptr [{rc} + 0x31A190]", "fmul dword ptr [{rc} + 0x1C5D00]", "fstp dword ptr [{rc} + 0x1CB984]", "fild dword ptr [{rc} + 0x1CB97C]", "fdiv dword ptr [{rc} + 0x1C5D14]", "fstp dword ptr [{rc} + 0x1CB97C]", "fild dword ptr [{rc} + 0x1CB980]", "fdiv dword ptr [{rc} + 0x1C5D14]", "fstp dword ptr [{rc} + 0x1CB980]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "test_drive_run line 42",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x1C5E34, In::F), (0x1CB968, In::F), (0x214748, In::F)],
                        writes: &[(0x214750, Out::F)],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0], a[1]) },
                plain: |a| x87_1(a[0], a[1]),
            },
            Block {
                name: "test_drive_run line 51",
                operands: vec![Operand::Buffer {
                    reads: &[
                        (0x1C5E34, In::F),
                        (0x21388C, In::F),
                        (0x213FCC, In::F),
                        (0x213FD0, In::F),
                        (0x214750, In::F),
                        (0x31A190, In::F),
                    ],
                    writes: &[(0x22400C, Out::F), (0x31A190, Out::F), (0x31A194, Out::F)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0]) },
                plain: |a| x87_2(a[0]),
            },
            Block {
                name: "test_drive_run line 93",
                operands: vec![Operand::Buffer {
                    reads: &[
                        (0x1C5D00, In::F),
                        (0x1C5D14, In::F),
                        (0x1CB97C, In::I),
                        (0x1CB980, In::I),
                        (0x31A190, In::F),
                    ],
                    writes: &[(0x1CB97C, Out::F), (0x1CB980, Out::F), (0x1CB984, Out::F)],
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
