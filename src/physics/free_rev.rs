//! `physics::free_rev` (0x43EA80): the engine revving free. `physics::engine_torque`;
//! `flt_614750` = `flt_614740` x `flt_5CB968` / gear 0's ratio / 10000 x +0x638 (`fild`) /
//! 256; the engine speed + it x the step / the ratio; the drag `dword_62400C` = engine
//! speed x `flt_613FD8` + `flt_613FD4`; then the engine speed brought toward 0 by the drag
//! (not past it; at 0 or unordered left alone).
//!
//! The x87 run is plain Rust over `X` (`math/x87_float.rs`) over the game's variables; the
//! original's instructions (from IDA) are `asm_reference` in the tests.
//!
//! In:  nothing
//! Out: as `engine_torque` leaves them; no flags read (the callers return or call on)
//!
//! Callers: `physics::free_rev_in_neutral`, `physics::free_rev_clutch`.

use crate::math::x87;
use crate::physics::car_store::CAR;
use crate::physics::engine_torque::{below, engine_torque};
use crate::scene::draw_store::DRAW;
use crate::scene::x87_mem::{fild, fld, fstp};

/// 0x43EA85 .. 0x43EAE7, as the original.
fn run() {
    x87_1(crate::race::store::asm_base() as usize);
}

/// The Rust version (the exact one, for `physics::free_rev_in_neutral`,
/// `free_rev_clutch` and their callers).
pub fn free_rev() {
    engine_torque();
    run();
    let (e, drag) = (CAR.engine_speed.bits(), DRAW.scratch.get());
    let x = f32::from_bits(e);
    if x == 0.0 || x.is_nan() {
        return;
    }
    if x < 0.0 {
        let y = x87::add(e, drag);
        CAR.engine_speed.set_bits(y);
        if !below(y, 0) {
            CAR.engine_speed.set_bits(0);
        }
    } else {
        let y = x87::sub(e, drag);
        CAR.engine_speed.set_bits(y);
        if below(y, 0) {
            CAR.engine_speed.set_bits(0);
        }
    }
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x214750] = (((([rc + 0x214740] * [rc + 0x1CB968]) / [rc + 0x2140C0]) / [rc + 0x1C5E34]) * int [rc + 0x31A610]) / [rc + 0x1C5E2C]`
/// `[rc + 0x31A1B0] = (([rc + 0x214750] * [rc + 0x21388C]) / [rc + 0x2140C0]) + [rc + 0x31A1B0]`
/// `[rc + 0x22400C] = ([rc + 0x31A1B0] * [rc + 0x213FD8]) + [rc + 0x213FD4]`
fn x87_1(rc: usize) {
    let t0 = fld(rc, 0x214740);
    let t1 = fld(rc, 0x1CB968);
    let t2 = t0 * t1;
    let t3 = fld(rc, 0x2140C0);
    let t4 = t2 / t3;
    let t5 = fld(rc, 0x1C5E34);
    let t6 = t4 / t5;
    let t7 = fild(rc, 0x31A610);
    let t8 = t6 * t7;
    let t9 = fld(rc, 0x1C5E2C);
    let t10 = t8 / t9;
    fstp(rc, 0x214750, t10);
    let t11 = fld(rc, 0x214750);
    let t12 = fld(rc, 0x21388C);
    let t13 = t11 * t12;
    let t14 = fld(rc, 0x2140C0);
    let t15 = t13 / t14;
    let t16 = fld(rc, 0x31A1B0);
    let t17 = t15 + t16;
    fstp(rc, 0x31A1B0, t17);
    let t18 = fld(rc, 0x31A1B0);
    let t19 = fld(rc, 0x213FD8);
    let t20 = t18 * t19;
    let t21 = fld(rc, 0x213FD4);
    let t22 = t20 + t21;
    fstp(rc, 0x22400C, t22);
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(rc) "fld dword ptr [{rc} + 0x214740]", "fmul dword ptr [{rc} + 0x1CB968]", "fdiv dword ptr [{rc} + 0x2140C0]", "fdiv dword ptr [{rc} + 0x1C5E34]", "fild dword ptr [{rc} + 0x31A610]", ".byte 0xDE, 0xC9", "fdiv dword ptr [{rc} + 0x1C5E2C]", "fstp dword ptr [{rc} + 0x214750]", "fld dword ptr [{rc} + 0x214750]", "fmul dword ptr [{rc} + 0x21388C]", "fdiv dword ptr [{rc} + 0x2140C0]", "fadd dword ptr [{rc} + 0x31A1B0]", "fstp dword ptr [{rc} + 0x31A1B0]", "fld dword ptr [{rc} + 0x31A1B0]", "fmul dword ptr [{rc} + 0x213FD8]", "fadd dword ptr [{rc} + 0x213FD4]", "fstp dword ptr [{rc} + 0x22400C]");

    fn blocks() -> Vec<Block> {
        vec![Block {
            name: "free_rev line 20",
            operands: vec![Operand::Buffer {
                reads: &[
                    (0x1C5E2C, In::F),
                    (0x1C5E34, In::F),
                    (0x1CB968, In::F),
                    (0x21388C, In::F),
                    (0x213FD4, In::F),
                    (0x213FD8, In::F),
                    (0x2140C0, In::F),
                    (0x214740, In::F),
                    (0x214750, In::F),
                    (0x31A1B0, In::F),
                    (0x31A610, In::I),
                ],
                writes: &[(0x214750, Out::F), (0x22400C, Out::F), (0x31A1B0, Out::F)],
            }],
            ulps: 0,
            reference: |a| unsafe { ref_0(a[0]) },
            plain: |a| x87_1(a[0]),
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
