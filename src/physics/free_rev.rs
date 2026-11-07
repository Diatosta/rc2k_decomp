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
use crate::math::x87_float::int;
use crate::math::x87_float::ld;
use crate::math::x87_float::st;
use crate::physics::engine_torque::{below, engine_torque};
use crate::physics::gear_store::GEAR;
use crate::race::owner::Race;
use crate::scene::owner::scene;

/// 0x43EA85 .. 0x43EAE7, as the original.
fn run(race: &Race) {
    x87_1(race);
}

/// The Rust version (the exact one, for `physics::free_rev_in_neutral`,
/// `free_rev_clutch` and their callers).
pub fn free_rev(race: &Race) {
    engine_torque(race);
    run(race);
    let (e, drag) = (
        race.shared.car.engine_speed.bits(),
        scene().drawing.scene.scratch.get(),
    );
    let x = f32::from_bits(e);
    if x == 0.0 || x.is_nan() {
        return;
    }
    if x < 0.0 {
        let y = x87::add(e, drag);
        race.shared.car.engine_speed.set_bits(y);
        if !below(y, 0) {
            race.shared.car.engine_speed.set_bits(0);
        }
    } else {
        let y = x87::sub(e, drag);
        race.shared.car.engine_speed.set_bits(y);
        if below(y, 0) {
            race.shared.car.engine_speed.set_bits(0);
        }
    }
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[rc + 0x214750] = (((([rc + 0x214740] * [rc + 0x1CB968]) / [rc + 0x2140C0]) / [rc + 0x1C5E34]) * int [rc + 0x31A610]) / [rc + 0x1C5E2C]`
/// `[rc + 0x31A1B0] = (([rc + 0x214750] * [rc + 0x21388C]) / [rc + 0x2140C0]) + [rc + 0x31A1B0]`
/// `[rc + 0x22400C] = ([rc + 0x31A1B0] * [rc + 0x213FD8]) + [rc + 0x213FD4]`
fn x87_1(race: &Race) {
    let t0 = ld(race.shared.physics_tables.drive_ratio.bits());
    let t1 = ld(GEAR.drive_torque_figure.bits());
    let t2 = t0 * t1;
    let t3 = ld(race.shared.physics_tables.gear_ratios[1].bits());
    let t4 = t2 / t3;
    let t5 = ld(race.course.numbers.k10000.bits());
    let t6 = t4 / t5;
    let t7 = int(race.shared.car.throttle.bits() as i32);
    let t8 = t6 * t7;
    let t9 = ld(race.course.numbers.k256.bits());
    let t10 = t8 / t9;
    race.shared.physics_tables.drive_torque.set_bits(st(t10));
    let t11 = ld(race.shared.physics_tables.drive_torque.bits());
    let t12 = ld(race.shared.physics_tables.step_time.bits());
    let t13 = t11 * t12;
    let t14 = ld(race.shared.physics_tables.gear_ratios[1].bits());
    let t15 = t13 / t14;
    let t16 = ld(race.shared.car.engine_speed.bits());
    let t17 = t15 + t16;
    race.shared.car.engine_speed.set_bits(st(t17));
    let t18 = ld(race.shared.car.engine_speed.bits());
    let t19 = ld(race.shared.physics_tables.free_rev_scale.bits());
    let t20 = t18 * t19;
    let t21 = ld(race.shared.physics_tables.free_rev_add.bits());
    let t22 = t20 + t21;
    scene().drawing.scene.scratch.set_bits(st(t22));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::race::owner::race;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(rc) "fld dword ptr [{rc} + 0x214740]", "fmul dword ptr [{rc} + 0x1CB968]", "fdiv dword ptr [{rc} + 0x2140C0]", "fdiv dword ptr [{rc} + 0x1C5E34]", "fild dword ptr [{rc} + 0x31A610]", ".byte 0xDE, 0xC9", "fdiv dword ptr [{rc} + 0x1C5E2C]", "fstp dword ptr [{rc} + 0x214750]", "fld dword ptr [{rc} + 0x214750]", "fmul dword ptr [{rc} + 0x21388C]", "fdiv dword ptr [{rc} + 0x2140C0]", "fadd dword ptr [{rc} + 0x31A1B0]", "fstp dword ptr [{rc} + 0x31A1B0]", "fld dword ptr [{rc} + 0x31A1B0]", "fmul dword ptr [{rc} + 0x213FD8]", "fadd dword ptr [{rc} + 0x213FD4]", "fstp dword ptr [{rc} + 0x22400C]");

    fn blocks() -> Vec<Block> {
        vec![Block {
            name: "free_rev line 20",
            operands: vec![Operand::RaceBlock {
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
            plain: |_| x87_1(race()),
        }]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
