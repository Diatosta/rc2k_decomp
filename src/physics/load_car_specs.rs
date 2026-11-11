//! `physics::load_car_specs` (0x421482): the work car's handling figures from its kind's
//! record (`g_car` +0x24, at most 0x21; records of 0xB0 at 0x5C790C, "00 renault maxi
//! megane" ..): the drive kind; four whole figures as floats into `figures` .. (each at
//! least 8.0) and the engine's top (+0x1DC, at least `thousand`); the shift points
//! `flt_5CB958` .. `shift_ratio_figure`; `drive_torque_figure`; `rev_limit`; the torque curve (nine
//! dwords) at `spec_torque`; the gear count (3 ..= 7, by the sign) and nine more words;
//! three whole figures x 65536 / `thousand` `fistp` into `spec_ints` .. (for kinds
//! below 0x17 from `spec_default_bc0` .. instead); `spec_grip` .. from `spec_default_ba0`,
//! `613F00`, `613F14` (the record's first written, then these); two floats x
//! `gravity`; `scratch`, `downforce_spec`; and by the drive kind `idle_engine_speed` and
//! it + 100 into `stall_engine_speed`.
//!
//! The x87 runs are plain Rust over `X` (`math/x87_float.rs`), the original's instructions
//! (from IDA) being `asm_reference` in the tests; the compares and the integer
//! steps in Rust.
//!
//! Callers: `physics::car_physics_step` (0x43B63A), `physics::test_drive`. Refuses (before
//! anything) a drive kind past 3 (`rev_limits` holds two floats; kinds 0 ..= 2 are read as the original does, kind 2 into
//! `camera_drop`).

use crate::errors::Refused;
use crate::math::x87;
use crate::math::x87_float::int;
use crate::math::x87_float::ld;
use crate::math::x87_float::st;
use crate::math::x87_float::to_int;
use crate::physics::car_store::Car;
use crate::physics::gear_store::GEAR;
use crate::physics::tables_store::PhysicsTables;
use crate::race::car_specs_store::CarSpec;
use crate::race::numbers_store::Numbers;
use crate::race::owner::Race;
use crate::session::{Cell32, Dword};
use rc2k_core::physics::car_specs::{OWN_FIGURES_FROM, check_drive, gear_count, record_of};
use std::cmp::Ordering;

/// `a` is below `b`, or one of them is not a number.
fn below(a: u32, b: u32) -> bool {
    !matches!(
        f32::from_bits(a).partial_cmp(&f32::from_bits(b)),
        Some(Ordering::Greater | Ordering::Equal)
    )
}

/// The record of the work car's kind (`record_of` holds it to the last).
fn record(race: &Race) -> &CarSpec {
    &race.physics.car_specs.cars[record_of(race.shared.car.car_kind.get()) as usize]
}

/// The whole-number steps both versions share, after the floats: the torque curve, the
/// gear count (held), the words copied.
fn words(tables: &PhysicsTables, spec: &CarSpec) {
    for (to, from) in tables.spec_torque.iter().zip(&spec.torque_curve) {
        to.set(from.get());
    }
    tables
        .ride_height_handling_scale
        .set(spec.ride_height_handling_scale.get());
    tables.gear_count.set(gear_count(spec.gear_count.get()));
    tables.unused_spec_20.set(spec.unused_20.get());
    let places: [(&dyn Cell32, &Dword); 5] = [
        (&tables.spec_surface[0], &spec.surface[0]),
        (&tables.spec_surface[1], &spec.surface[1]),
        (&tables.grip_scale, &spec.grip_scale),
        (&tables.side_grip_scale, &spec.side_grip_scale),
        (&tables.spec_surface[2], &spec.surface[2]),
    ];
    for (to, from) in places {
        to.set_bits(from.get());
    }
}

/// The exact version, for `physics::car_physics_step` and `physics::test_drive`.
pub fn load_car_specs(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let course = &race.course;
    let kind = race.shared.car.car_kind.get();
    let spec = record(race);
    let drive = check_drive(spec.drive_kind.get())?;
    race.shared.car.drive_kind.set(drive);
    // 0x4214A7 .. 0x421543
    load_figures_and_shift_points(
        &race.shared.car,
        &race.shared.physics_tables,
        &race.course.numbers,
        spec,
    );
    if below(
        race.shared.car.engine_speed_limit.bits(),
        course.numbers.thousand.bits(),
    ) {
        race.shared
            .car
            .engine_speed_limit
            .set(x87::loaded(&course.numbers.thousand));
    }
    for figure in &GEAR.figures {
        if below(figure.bits(), course.numbers.eight.bits()) {
            figure.set(x87::loaded(&course.numbers.eight));
        }
    }
    words(&race.shared.physics_tables, spec);
    // 0x4216C5 .. 0x421704
    scale_record_ratios(&race.shared.physics_tables, &race.course.numbers, spec);
    if kind < OWN_FIGURES_FROM {
        // 0x42170D .. 0x421755
        scale_default_ratios(&race.shared.physics_tables, &race.course.numbers);
    }
    race.shared
        .physics_tables
        .spec_grip
        .set_bits(race.shared.physics_tables.spec_default_ba0.get());
    race.shared
        .physics_tables
        .spec_grip_b
        .set_bits(race.shared.physics_tables.spec_default_ba4.get());
    race.shared
        .physics_tables
        .spec_wheel
        .set_bits(race.shared.physics_tables.spec_default_ba8.get());
    // 0x421794 .. 0x4217B2
    scale_record_grips(&race.shared.physics_tables, spec);
    scene.drawing.scene.scratch.set(spec.scratch_value.get());
    race.shared
        .physics_tables
        .downforce_spec
        .set(spec.downforce.get());
    // 0x4217CA .. 0x4217E3
    set_rev_limits(
        &race.shared.physics_tables,
        &race.course.numbers,
        drive as usize,
    );
    Ok(())
}

// ---- the arithmetic that was x87 assembly ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[figures] = int [s + 0x64]`
/// `[rc + 0x1CB970] = int [s + 0x68]`
/// `[rc + 0x1CB974] = int [s + 0x6C]`
/// `[rc + 0x1CB978] = int [s + 0x70]`
/// `[engine_speed_limit] = int [s + 0x74]`
/// `[spec_scale_copy] = [spec_scale_1b8]`
/// `[rc + 0x1CB958] = [s + 0xA4]`
/// `[shift_ratio_figure] = [s + 0xA4] * [spec_scale_a4]`
/// `[shift_down_point] = [s + 0xA8] + [tenths]`
/// `[shift_down_clutch_point] = [s + 0xAC] + [tenths]`
/// `[drive_torque_figure] = (int [s + 0x78] * [rev_limit_scale]) / [engine_speed_limit]`
/// `[rev_limit] = [engine_speed_limit] * [rev_top_scale]`
fn load_figures_and_shift_points(
    car: &Car,
    tables: &PhysicsTables,
    numbers: &Numbers,
    s: &CarSpec,
) {
    let t0 = int(s.figures[0].get() as i32);
    GEAR.figures[0].set_bits(st(t0));
    let t1 = int(s.figures[1].get() as i32);
    GEAR.figures[1].set_bits(st(t1));
    let t2 = int(s.figures[2].get() as i32);
    GEAR.figures[2].set_bits(st(t2));
    let t3 = int(s.figures[3].get() as i32);
    GEAR.figures[3].set_bits(st(t3));
    let t4 = int(s.engine_top.get() as i32);
    car.engine_speed_limit.set_bits(st(t4));
    let t5 = ld(tables.spec_scale_1b8.bits());
    car.spec_scale_copy.set_bits(st(t5));
    let t6 = ld(s.shift_points[0].bits());
    GEAR.shift_up_point.set_bits(st(t6));
    let t7 = ld(tables.spec_scale_a4.bits());
    let t8 = t6 * t7;
    GEAR.shift_ratio_figure.set_bits(st(t8));
    let t9 = ld(s.shift_points[1].bits());
    let t10 = ld(numbers.tenths[0].bits());
    let t11 = t9 + t10;
    GEAR.shift_down_point.set_bits(st(t11));
    let t12 = ld(s.shift_points[2].bits());
    let t13 = ld(numbers.tenths[0].bits());
    let t14 = t12 + t13;
    GEAR.shift_down_clutch_point.set_bits(st(t14));
    let t15 = int(s.over_top.get() as i32);
    let t16 = ld(tables.rev_limit_scale.bits());
    let t17 = t15 * t16;
    let t18 = ld(car.engine_speed_limit.bits());
    let t19 = t17 / t18;
    GEAR.drive_torque_figure.set_bits(st(t19));
    let t20 = ld(car.engine_speed_limit.bits());
    let t21 = ld(tables.rev_top_scale.bits());
    let t22 = t20 * t21;
    tables.rev_limit.set_bits(st(t22));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [spec_ints] = (int [s + 0x40] * [k65536]) / [thousand]`
/// `int [rc + 0x214BC4] = (int [s + 0x44] * [k65536]) / [thousand]`
/// `int [rc + 0x214BC8] = (int [s + 0x48] * [k65536]) / [thousand]`
fn scale_record_ratios(tables: &PhysicsTables, numbers: &Numbers, s: &CarSpec) {
    let t0 = int(s.whole_figures[0].get() as i32);
    let t1 = ld(numbers.k65536.bits());
    let t2 = t0 * t1;
    let t3 = ld(numbers.thousand.bits());
    let t4 = t2 / t3;
    tables.spec_ints[0].set_bits(to_int(t4) as u32);
    let t5 = int(s.whole_figures[1].get() as i32);
    let t6 = ld(numbers.k65536.bits());
    let t7 = t5 * t6;
    let t8 = ld(numbers.thousand.bits());
    let t9 = t7 / t8;
    tables.spec_ints[1].set_bits(to_int(t9) as u32);
    let t10 = int(s.whole_figures[2].get() as i32);
    let t11 = ld(numbers.k65536.bits());
    let t12 = t10 * t11;
    let t13 = ld(numbers.thousand.bits());
    let t14 = t12 / t13;
    tables.spec_ints[2].set_bits(to_int(t14) as u32);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [spec_ints] = (int [spec_default_bc0] * [k65536]) / [thousand]`
/// `int [rc + 0x214BC4] = (int [rc + 0x213F08] * [k65536]) / [thousand]`
/// `int [rc + 0x214BC8] = (int [rc + 0x213F0C] * [k65536]) / [thousand]`
fn scale_default_ratios(tables: &PhysicsTables, numbers: &Numbers) {
    let t0 = int(tables.spec_default_bc0[0].bits() as i32);
    let t1 = ld(numbers.k65536.bits());
    let t2 = t0 * t1;
    let t3 = ld(numbers.thousand.bits());
    let t4 = t2 / t3;
    tables.spec_ints[0].set_bits(to_int(t4) as u32);
    let t5 = int(tables.spec_default_bc0[1].bits() as i32);
    let t6 = ld(numbers.k65536.bits());
    let t7 = t5 * t6;
    let t8 = ld(numbers.thousand.bits());
    let t9 = t7 / t8;
    tables.spec_ints[1].set_bits(to_int(t9) as u32);
    let t10 = int(tables.spec_default_bc0[2].bits() as i32);
    let t11 = ld(numbers.k65536.bits());
    let t12 = t10 * t11;
    let t13 = ld(numbers.thousand.bits());
    let t14 = t12 / t13;
    tables.spec_ints[2].set_bits(to_int(t14) as u32);
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[spec_drive] = [s + 0x38] * [gravity]`
/// `[rc + 0x214BB0] = [s + 0x3C] * [gravity]`
fn scale_record_grips(tables: &PhysicsTables, s: &CarSpec) {
    let t0 = ld(s.weights[0].bits());
    let t1 = ld(tables.gravity.bits());
    let t2 = t0 * t1;
    tables.spec_drive[0].set_bits(st(t2));
    let t3 = ld(s.weights[1].bits());
    let t4 = ld(tables.gravity.bits());
    let t5 = t3 * t4;
    tables.spec_drive[1].set_bits(st(t5));
}

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `[idle_engine_speed] = [rc + k * 4 + 0x213FB8]`
/// `[stall_engine_speed] = [rc + k * 4 + 0x213FB8] + [hundred]`
/// `k` is the drive kind (0 .. 2): kind 2 reads past the two rev limits into `camera_drop`,
/// as the original (`notes/bugs.md`).
fn set_rev_limits(tables: &PhysicsTables, numbers: &Numbers, k: usize) {
    let limit = [
        &tables.rev_limits[0],
        &tables.rev_limits[1],
        &tables.camera_drop,
    ][k];
    let t0 = ld(limit.bits());
    tables.idle_engine_speed.set_bits(st(t0));
    let t1 = ld(numbers.hundred.bits());
    let t2 = t0 + t1;
    tables.stall_engine_speed.set_bits(st(t2));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::race::owner::race;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    /// A record with the bytes `from .. to` of the test buffer at `base` (the rest zero):
    /// the buffers hold only the offsets the run reads.
    fn spec_at(base: usize, from: usize, to: usize) -> Box<CarSpec> {
        // SAFETY: cells and byte arrays only, all-zero is valid; the buffer holds `to` bytes.
        let spec: Box<CarSpec> = Box::new(unsafe { std::mem::zeroed() });
        unsafe {
            std::ptr::copy_nonoverlapping(
                crate::pointers::host_at(base + from),
                std::ptr::from_ref::<CarSpec>(&spec)
                    .cast::<u8>()
                    .cast_mut()
                    .add(from),
                to - from,
            );
        }
        spec
    }

    // The original's instructions, one function per run.
    reference!(ref_0(s, rc) "fild dword ptr [{s} + 0x64]", "fstp dword ptr [{rc} + 0x1CB96C]", "fild dword ptr [{s} + 0x68]", "fstp dword ptr [{rc} + 0x1CB970]", "fild dword ptr [{s} + 0x6C]", "fstp dword ptr [{rc} + 0x1CB974]", "fild dword ptr [{s} + 0x70]", "fstp dword ptr [{rc} + 0x1CB978]", "fild dword ptr [{s} + 0x74]", "fstp dword ptr [{rc} + 0x31A1B4]", "fld dword ptr [{rc} + 0x213F10]", "fstp dword ptr [{rc} + 0x31A1B8]", "fld dword ptr [{s} + 0xA4]", "fst dword ptr [{rc} + 0x1CB958]", "fmul dword ptr [{rc} + 0x213E88]", "fstp dword ptr [{rc} + 0x1CB964]", "fld dword ptr [{s} + 0xA8]", "fadd dword ptr [{rc} + 0x1C5DDC]", "fstp dword ptr [{rc} + 0x1CB95C]", "fld dword ptr [{s} + 0xAC]", "fadd dword ptr [{rc} + 0x1C5DDC]", "fstp dword ptr [{rc} + 0x1CB960]", "fild dword ptr [{s} + 0x78]", "fmul dword ptr [{rc} + 0x213EB8]", "fdiv dword ptr [{rc} + 0x31A1B4]", "fstp dword ptr [{rc} + 0x1CB968]", "fld dword ptr [{rc} + 0x31A1B4]", "fmul dword ptr [{rc} + 0x213FE4]", "fstp dword ptr [{rc} + 0x214104]");
    reference!(ref_1(s, rc) "fild dword ptr [{s} + 0x40]", "fmul dword ptr [{rc} + 0x1C5D98]", "fdiv dword ptr [{rc} + 0x1C5D18]", "fistp dword ptr [{rc} + 0x214BC0]", "fild dword ptr [{s} + 0x44]", "fmul dword ptr [{rc} + 0x1C5D98]", "fdiv dword ptr [{rc} + 0x1C5D18]", "fistp dword ptr [{rc} + 0x214BC4]", "fild dword ptr [{s} + 0x48]", "fmul dword ptr [{rc} + 0x1C5D98]", "fdiv dword ptr [{rc} + 0x1C5D18]", "fistp dword ptr [{rc} + 0x214BC8]");
    reference!(ref_2(rc) "fild dword ptr [{rc} + 0x213F04]", "fmul dword ptr [{rc} + 0x1C5D98]", "fdiv dword ptr [{rc} + 0x1C5D18]", "fistp dword ptr [{rc} + 0x214BC0]", "fild dword ptr [{rc} + 0x213F08]", "fmul dword ptr [{rc} + 0x1C5D98]", "fdiv dword ptr [{rc} + 0x1C5D18]", "fistp dword ptr [{rc} + 0x214BC4]", "fild dword ptr [{rc} + 0x213F0C]", "fmul dword ptr [{rc} + 0x1C5D98]", "fdiv dword ptr [{rc} + 0x1C5D18]", "fistp dword ptr [{rc} + 0x214BC8]");
    reference!(ref_3(s, rc) "fld dword ptr [{s} + 0x38]", "fmul dword ptr [{rc} + 0x213FC8]", "fstp dword ptr [{rc} + 0x214BAC]", "fld dword ptr [{s} + 0x3C]", "fmul dword ptr [{rc} + 0x213FC8]", "fstp dword ptr [{rc} + 0x214BB0]");
    reference!(ref_4(rc, k) "fld dword ptr [{rc} + {k}*4 + 0x213FB8]", "fst dword ptr [{rc} + 0x2140F8]", "fadd dword ptr [{rc} + 0x1C5D14]", "fstp dword ptr [{rc} + 0x2140FC]");

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "load_car_specs line 100",
                operands: vec![
                    Operand::Buffer {
                        reads: &[
                            (0x64, In::I),
                            (0x68, In::I),
                            (0x6C, In::I),
                            (0x70, In::I),
                            (0x74, In::I),
                            (0x78, In::I),
                            (0xA4, In::F),
                            (0xA8, In::F),
                            (0xAC, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[
                            (0x1C5DDC, In::F),
                            (0x213E88, In::F),
                            (0x213EB8, In::F),
                            (0x213F10, In::F),
                            (0x213FE4, In::F),
                            (0x31A1B4, In::F),
                        ],
                        writes: &[
                            (0x1CB958, Out::F),
                            (0x1CB95C, Out::F),
                            (0x1CB960, Out::F),
                            (0x1CB964, Out::F),
                            (0x1CB968, Out::F),
                            (0x1CB96C, Out::F),
                            (0x1CB970, Out::F),
                            (0x1CB974, Out::F),
                            (0x1CB978, Out::F),
                            (0x214104, Out::F),
                            (0x31A1B4, Out::F),
                            (0x31A1B8, Out::F),
                        ],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_0(a[0], a[1]) },
                plain: |a| {
                    load_figures_and_shift_points(
                        &race().shared.car,
                        &race().shared.physics_tables,
                        &race().course.numbers,
                        &spec_at(a[0], 0x64, 0xB0),
                    );
                },
            },
            Block {
                name: "load_car_specs line 145",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x40, In::I), (0x44, In::I), (0x48, In::I)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x1C5D18, In::F), (0x1C5D98, In::F)],
                        writes: &[(0x214BC0, Out::I), (0x214BC4, Out::I), (0x214BC8, Out::I)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_1(a[0], a[1]) },
                plain: |a| {
                    scale_record_ratios(
                        &race().shared.physics_tables,
                        &race().course.numbers,
                        &spec_at(a[0], 0x40, 0x4C),
                    );
                },
            },
            Block {
                name: "load_car_specs line 162",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x1C5D18, In::F),
                        (0x1C5D98, In::F),
                        (0x213F04, In::I),
                        (0x213F08, In::I),
                        (0x213F0C, In::I),
                    ],
                    writes: &[(0x214BC0, Out::I), (0x214BC4, Out::I), (0x214BC8, Out::I)],
                }],
                ulps: 0,
                reference: |a| unsafe { ref_2(a[0]) },
                plain: |_| {
                    scale_default_ratios(&race().shared.physics_tables, &race().course.numbers);
                },
            },
            Block {
                name: "load_car_specs line 182",
                operands: vec![
                    Operand::Buffer {
                        reads: &[(0x38, In::F), (0x3C, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[(0x213FC8, In::F)],
                        writes: &[(0x214BAC, Out::F), (0x214BB0, Out::F)],
                    },
                ],
                ulps: 0,
                reference: |a| unsafe { ref_3(a[0], a[1]) },
                plain: |a| {
                    scale_record_grips(&race().shared.physics_tables, &spec_at(a[0], 0x38, 0x40));
                },
            },
            Block {
                name: "load_car_specs line 194 idx k=0",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[(0x1C5D14, In::F), (0x213FB8, In::F)],
                        writes: &[(0x2140F8, Out::F), (0x2140FC, Out::F)],
                    },
                    Operand::Value(0),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_4(a[0], a[1]) },
                plain: |a| {
                    set_rev_limits(&race().shared.physics_tables, &race().course.numbers, a[1]);
                },
            },
            Block {
                name: "load_car_specs line 194 idx k=1",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[(0x1C5D14, In::F), (0x213FBC, In::F)],
                        writes: &[(0x2140F8, Out::F), (0x2140FC, Out::F)],
                    },
                    Operand::Value(1),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_4(a[0], a[1]) },
                plain: |a| {
                    set_rev_limits(&race().shared.physics_tables, &race().course.numbers, a[1]);
                },
            },
            Block {
                name: "load_car_specs line 194 idx k=2 (camera_drop)",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[(0x1C5D14, In::F), (0x213FC0, In::F)],
                        writes: &[(0x2140F8, Out::F), (0x2140FC, Out::F)],
                    },
                    Operand::Value(2),
                ],
                ulps: 0,
                reference: |a| unsafe { ref_4(a[0], a[1]) },
                plain: |a| {
                    set_rev_limits(&race().shared.physics_tables, &race().course.numbers, a[1]);
                },
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
