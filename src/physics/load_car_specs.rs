//! The work car's handling figures from its kind's record (kind at most 0x21; records of 0xB0
//! bytes, the first named "00 renault maxi megane"): the drive kind; four whole figures as floats
//! into `figures` (each at least 8.0) and the engine's top (at least `thousand`); the shift points
//! `LightsAndGear.gear.shift_up_point` .. `shift_ratio_figure`; `drive_torque_figure`; `rev_limit`;
//! the torque curve (nine dwords) at `spec_torque`; the gear count (3 ..= 7) and nine more words;
//! three whole figures x 65536 / `thousand`, rounded, into `spec_ints` (for kinds below 0x17 from
//! `spec_default_bc0` instead); `spec_grip` from `spec_default_ba0` and two more defaults (the
//! record's first written, then these); two floats x `gravity`; `scratch`, `downforce_spec`; and by
//! the drive kind `idle_engine_speed` and it + 100 into `stall_engine_speed`.
//!
//! The x87 runs are plain Rust over `X` (`math/x87_float.rs`); the compares and the integer steps
//! are Rust.
//!
//! Refuses (before anything) a drive kind past 3 (`rev_limits` holds two floats; kinds 0 ..= 2 are
//! read, kind 2 into `camera_drop`).

use crate::effects::cell_x87::CellX87;
use crate::errors::Refused;
use crate::math::x87;
use crate::math::x87_float::int;
use crate::math::x87_float::to_int;
use crate::physics::car_store::Car;
use crate::physics::gear_store::GearFigures;
use crate::physics::tables_store::PhysicsTables;
use crate::race::car_specs_store::CarSpec;
use crate::race::car_specs_store::CarSpecs;
use crate::race::numbers_store::Numbers;
use crate::race::owner_course::Course;
use rc2k_core::physics::car_specs::{OWN_FIGURES_FROM, check_drive, gear_count, record_of};
use std::cmp::Ordering;

/// `a` is below `b`, or one of them is not a number.
fn below(a: f32, b: f32) -> bool {
    !matches!(a.partial_cmp(&b), Some(Ordering::Greater | Ordering::Equal))
}

/// The record of the work car's kind (`record_of` holds it to the last).
fn record<'a>(car_specs: &'a CarSpecs, car: &Car) -> &'a CarSpec {
    &car_specs.cars[record_of(car.car_kind.get()) as usize]
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
    tables.spec_surface[0].set(spec.surface[0].get());
    tables.spec_surface[1].set(spec.surface[1].get());
    tables.grip_scale.set(spec.grip_scale.get());
    tables.side_grip_scale.set(spec.side_grip_scale.get());
    tables.spec_surface[2].set(spec.surface[2].get());
}

/// The exact version, for `physics::car_physics_step` and `physics::test_drive`.
pub fn load_car_specs(
    car_specs: &CarSpecs,
    course: &Course,
    physics_tables: &PhysicsTables,
    car: &Car,
    gear_figures: &GearFigures,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let kind = car.car_kind.get();
    let spec = record(car_specs, car);
    let drive = check_drive(spec.drive_kind.get())?;
    car.drive_kind.set(drive);
    load_figures_and_shift_points(
        car,
        physics_tables,
        course.numbers.get(),
        gear_figures,
        spec,
    );
    if below(
        car.engine_speed_limit.get(),
        course.numbers.get().thousand.get(),
    ) {
        car.engine_speed_limit
            .set(x87::loaded(&course.numbers.get().thousand));
    }
    for figure in &gear_figures.figures {
        if below(figure.get(), course.numbers.get().eight.get()) {
            figure.set(x87::loaded(&course.numbers.get().eight));
        }
    }
    words(physics_tables, spec);
    scale_record_ratios(physics_tables, course.numbers.get(), spec);
    if kind < OWN_FIGURES_FROM {
        scale_default_ratios(physics_tables, course.numbers.get());
    }
    physics_tables
        .spec_grip
        .set(physics_tables.spec_default_ba0.get());
    physics_tables
        .spec_grip_b
        .set(physics_tables.spec_default_ba4.get());
    physics_tables
        .spec_wheel
        .set(physics_tables.spec_default_ba8.get());
    scale_record_grips(physics_tables, spec);
    scene
        .drawing
        .scene
        .get()
        .scratch
        .set(spec.scratch_value.get());
    physics_tables.downforce_spec.set(spec.downforce.get());
    set_rev_limits(physics_tables, course.numbers.get(), drive as usize);
    Ok(())
}

// ---- the x87 arithmetic ----

/// One straight x87 run as plain Rust over `X`.
fn load_figures_and_shift_points(
    car: &Car,
    tables: &PhysicsTables,
    numbers: &Numbers,
    gear_figures: &GearFigures,
    s: &CarSpec,
) {
    let t0 = int(s.figures[0].get() as i32);
    gear_figures.figures[0].st(t0);
    let t1 = int(s.figures[1].get() as i32);
    gear_figures.figures[1].st(t1);
    let t2 = int(s.figures[2].get() as i32);
    gear_figures.figures[2].st(t2);
    let t3 = int(s.figures[3].get() as i32);
    gear_figures.figures[3].st(t3);
    let t4 = int(s.engine_top.get() as i32);
    car.engine_speed_limit.st(t4);
    let t5 = tables.spec_scale_1b8.ld();
    car.spec_scale_copy.st(t5);
    let t6 = s.shift_points[0].ld();
    gear_figures.shift_up_point.st(t6);
    let t7 = tables.spec_scale_a4.ld();
    let t8 = t6 * t7;
    gear_figures.shift_ratio_figure.st(t8);
    let t9 = s.shift_points[1].ld();
    let t10 = numbers.tenths[0].ld();
    let t11 = t9 + t10;
    gear_figures.shift_down_point.st(t11);
    let t12 = s.shift_points[2].ld();
    let t13 = numbers.tenths[0].ld();
    let t14 = t12 + t13;
    gear_figures.shift_down_clutch_point.st(t14);
    let t15 = int(s.over_top.get() as i32);
    let t16 = tables.rev_limit_scale.ld();
    let t17 = t15 * t16;
    let t18 = car.engine_speed_limit.ld();
    let t19 = t17 / t18;
    gear_figures.drive_torque_figure.st(t19);
    let t20 = car.engine_speed_limit.ld();
    let t21 = tables.rev_top_scale.ld();
    let t22 = t20 * t21;
    tables.rev_limit.st(t22);
}

/// One straight x87 run as plain Rust over `X`.
fn scale_record_ratios(tables: &PhysicsTables, numbers: &Numbers, s: &CarSpec) {
    let t0 = int(s.whole_figures[0].get() as i32);
    let t1 = numbers.k65536.ld();
    let t2 = t0 * t1;
    let t3 = numbers.thousand.ld();
    let t4 = t2 / t3;
    tables.spec_ints[0].set(to_int(t4) as u32);
    let t5 = int(s.whole_figures[1].get() as i32);
    let t6 = numbers.k65536.ld();
    let t7 = t5 * t6;
    let t8 = numbers.thousand.ld();
    let t9 = t7 / t8;
    tables.spec_ints[1].set(to_int(t9) as u32);
    let t10 = int(s.whole_figures[2].get() as i32);
    let t11 = numbers.k65536.ld();
    let t12 = t10 * t11;
    let t13 = numbers.thousand.ld();
    let t14 = t12 / t13;
    tables.spec_ints[2].set(to_int(t14) as u32);
}

/// One straight x87 run as plain Rust over `X`.
fn scale_default_ratios(tables: &PhysicsTables, numbers: &Numbers) {
    let t0 = int(tables.spec_default_bc0[0].get() as i32);
    let t1 = numbers.k65536.ld();
    let t2 = t0 * t1;
    let t3 = numbers.thousand.ld();
    let t4 = t2 / t3;
    tables.spec_ints[0].set(to_int(t4) as u32);
    let t5 = int(tables.spec_default_bc0[1].get() as i32);
    let t6 = numbers.k65536.ld();
    let t7 = t5 * t6;
    let t8 = numbers.thousand.ld();
    let t9 = t7 / t8;
    tables.spec_ints[1].set(to_int(t9) as u32);
    let t10 = int(tables.spec_default_bc0[2].get() as i32);
    let t11 = numbers.k65536.ld();
    let t12 = t10 * t11;
    let t13 = numbers.thousand.ld();
    let t14 = t12 / t13;
    tables.spec_ints[2].set(to_int(t14) as u32);
}

/// One straight x87 run as plain Rust over `X`.
fn scale_record_grips(tables: &PhysicsTables, s: &CarSpec) {
    let t0 = s.weights[0].ld();
    let t1 = tables.gravity.ld();
    let t2 = t0 * t1;
    tables.spec_drive[0].st(t2);
    let t3 = s.weights[1].ld();
    let t4 = tables.gravity.ld();
    let t5 = t3 * t4;
    tables.spec_drive[1].st(t5);
}

/// `k` is the drive kind (0 .. 2): kind 2 reads past the two rev limits into `camera_drop` (a slip
/// kept as is).
fn set_rev_limits(tables: &PhysicsTables, numbers: &Numbers, k: usize) {
    let limit = [
        &tables.rev_limits[0],
        &tables.rev_limits[1],
        &tables.camera_drop,
    ][k];
    let t0 = limit.ld();
    tables.idle_engine_speed.st(t0);
    let t1 = numbers.hundred.ld();
    let t2 = t0 + t1;
    tables.stall_engine_speed.st(t2);
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::own::race;
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    /// A record with the bytes `from .. to` of the test buffer at `base` (the rest zero):
    /// the buffers hold only the offsets the run reads.
    fn spec_at(base: usize, from: usize, to: usize) -> Box<CarSpec> {
        // SAFETY: cells and byte arrays only, all-zero is valid; the buffer holds `to` bytes.
        let mut spec: Box<CarSpec> = Box::new(unsafe { std::mem::zeroed() });
        use crate::test_support::vectors::block_vectors::{LAID_ROOM, at};
        assert!(from <= to && to <= size_of::<CarSpec>() && to <= LAID_ROOM);
        // SAFETY: the buffer at `base` holds at least `LAID_ROOM` bytes (`block_vectors`), so
        // `from .. to` (asserted inside it) lies in it; the record is this function's own box,
        // written through its unique pointer inside its size (asserted), any bytes valid for its
        // cells and byte arrays; the two do not overlap.
        unsafe {
            std::ptr::copy_nonoverlapping(
                at(base + from),
                (&raw mut *spec).cast::<u8>().add(from),
                to - from,
            );
        }
        spec
    }

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
                plain: |a| {
                    load_figures_and_shift_points(
                        race().shared.car(),
                        race().shared.physics_tables.get(),
                        race().course.numbers.get(),
                        race().gear(),
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
                plain: |a| {
                    scale_record_ratios(
                        race().shared.physics_tables.get(),
                        race().course.numbers.get(),
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
                plain: |_| {
                    scale_default_ratios(
                        race().shared.physics_tables.get(),
                        race().course.numbers.get(),
                    );
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
                plain: |a| {
                    scale_record_grips(
                        race().shared.physics_tables.get(),
                        &spec_at(a[0], 0x38, 0x40),
                    );
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
                plain: |a| {
                    set_rev_limits(
                        race().shared.physics_tables.get(),
                        race().course.numbers.get(),
                        a[1],
                    );
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
                plain: |a| {
                    set_rev_limits(
                        race().shared.physics_tables.get(),
                        race().course.numbers.get(),
                        a[1],
                    );
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
                plain: |a| {
                    set_rev_limits(
                        race().shared.physics_tables.get(),
                        race().course.numbers.get(),
                        a[1],
                    );
                },
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("physics.load_car_specs", &blocks(), 1200, 100);
    }
}
