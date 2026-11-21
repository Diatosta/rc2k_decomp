//! The body's deepest point against the ground. `contact_found` = 0 and
//! `physics::lowest_body_point` runs; unless the depth (`lowest_depth`) is below `crash_depth`
//! and, with the wheels on the ground, below 0, only `physics::sound_cue_e5` runs. Otherwise
//! the point `lowest_point` gives (from `probe_x`, `probe_ground_height`, `probe_z`) becomes
//! the car's contact point and `contact_found` = 1; the contact is taken (`take_contact`: the
//! car's speed, the face's normal, the turn speed, the point); the car's up vector (0, 1, 0
//! turned by the crash matrix) is dotted with the normal (negated), and from that comes the slide limit
//! `contact_slide_limit`;
//! `physics::contact_force` runs into force record 21, then `physics::place_sound_e5`; the
//! force goes into `ground_force` and the record's point is 0; the ground scale =
//! (wheel-load figure + 64) / 64 held at most 1, times the car's spin figure; the car's
//! `contact_value` is the contact work's.
//!
//! Refuses a point past 0x3F on the long path (the tables' room), and what its calls refuse; what
//! was written is put back.

use crate::errors::Refused;
use crate::math::cell_x87::CellX87;
use crate::math::matrix3::{extended, rounded_value};
use crate::math::x87::neg_rounded;
use crate::math::x87::{loaded, mul};
use crate::memory::session::{FloatLike, PlainFloat};
use crate::physics::car_store::Car;
use crate::physics::contact_force::contact_force;
use crate::physics::contact_store::ContactWork;
use crate::physics::dot_product::dot_product;
use crate::physics::flow::FloatView;
use crate::physics::flow::hold_high;
use crate::physics::lowest_body_point::lowest_body_point;
use crate::physics::place_sound_e5::place_sound_e5;
use crate::physics::sound_cue_e5::sound_cue_e5;
use crate::physics::step_families::ContactInputs;
use crate::physics::tables_store::PhysicsTables;
use crate::race::numbers_store::Numbers;
use crate::race::owner_course::Course;
use crate::scene::rotate_point::rotate_point;
use rc2k_core::physics::contacts;

/// Three floats copied as bits, in order.
fn copy3(to: &[impl FloatLike], from: &[impl FloatLike]) {
    for k in 0..3 {
        to[k].set(from[k].get());
    }
}

/// The sign of a comparison's difference.
pub fn sign(v: u32) -> bool {
    (v as i32) < 0
}

/// The contact of point `n` taken: the car's `velocity` goes into `contact_car_speed`; the
/// normal of its face (`probe_point_face[n]` in the course faces: its first normal, or with
/// `probe_face_flip[n]` at 1 its second) into `contact_normal`; the car's `spin` into
/// `contact_turn_speed`; the point (`probe_turned_x[n]`, `probe_turned_y[n]`, `probe_turned_z[n]`)
/// into `contact_point`.
pub(super) fn take_contact(
    contact: &ContactWork,
    course: &Course,
    car: &Car,
    n: usize,
) -> Result<(), Refused> {
    copy3(&contact.contact_car_speed, &car.velocity);
    let face = car.probe_point_face[n].get();
    let parts = contacts::normal_offsets(contact.probe_face_flip[n].get());
    for (k, &o) in parts.iter().enumerate() {
        let v = crate::race::course_places::face_field::<u32>(course.reads(), face, o)?;
        contact.contact_normal[k].set(loaded(v));
    }
    copy3(&contact.contact_turn_speed, &car.spin);
    contact.contact_point[0].set(contact.probe_turned_x[n].get());
    contact.contact_point[1].set(contact.probe_turned_y[n].get());
    contact.contact_point[2].set(contact.probe_turned_z[n].get());
    Ok(())
}

/// The deepest point's number (`lowest_point`) for the long path; refused past 0x3F.
pub fn point_number(contact: &ContactWork) -> Result<usize, Refused> {
    let n = contact.lowest_point.get();
    contacts::check_body_point(n)?;
    Ok(n as usize)
}

// Core: `rc2k_core::physics::contacts::{touches, normal_offsets, check_body_point}`
// (the x87 steps stay).
fn steps(c: &ContactInputs<'_>) -> Result<(), Refused> {
    c.car.contact_found.set(0);
    { lowest_body_point(c.car, c.numbers, c.contact) }?;
    let depth = c.contact.lowest_depth.get();
    if !contacts::touches(
        depth,
        c.tables.crash_depth.get(),
        c.car.all_wheels_off.get() != 0,
    ) {
        { sound_cue_e5(c.sound.frame_work, c.work, c.sound.owner) }?;
        return Ok(());
    }
    let n = point_number(c.contact)?;
    c.car.contact_point_x.set(c.contact.probe_x[n].get());
    c.car
        .contact_point_ground
        .set(c.contact.probe_ground_height[n].get());
    c.car.contact_point_z.set(c.contact.probe_z[n].get());
    c.car.contact_found.set(1);
    take_contact(c.contact, c.course, c.car, n)?;
    let up = {
        rotate_point(
            &c.car.crash_matrix,
            [extended(0), extended(0x3F80_0000), extended(0)],
        )
    };
    // Made and read within this contact.
    let car_up = PlainFloat::array([0.0; 3]);
    let up_dot = PlainFloat::new(0.0);
    let ground_scale = PlainFloat::new(0.0);
    for (k, e) in up.iter().enumerate() {
        car_up[k].set(rounded_value(*e));
    }
    up_dot.set_float(neg_rounded(dot_product(&car_up, &c.contact.contact_normal)));
    x87_1(&up_dot, c.contact, c.tables, c.numbers);
    contact_force(c.contact, c.numbers, c.tables, c.car, &c.tables.forces[21]);
    { place_sound_e5(&c.sound) }?;
    for k in 0..3 {
        c.contact.ground_force[k].set_float(c.tables.forces[21].force[k].get());
    }
    for cell in &c.tables.forces[21].point {
        cell.set(0.0);
    }
    x87_2(&ground_scale, c.car, c.numbers);
    hold_high(&ground_scale, &c.numbers.one);
    c.car.spin[1].set(mul(&ground_scale, &c.car.spin[1]));
    c.car.contact_value.set_float(c.contact.contact_value.get());
    Ok(())
}

pub fn crash_contact(c: &ContactInputs<'_>) -> Result<(), Refused> {
    steps(c)
}

// ---- the x87 arithmetic ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// A long formula, covered by the tests.
fn x87_1(up_dot: &PlainFloat, contact: &ContactWork, tables: &PhysicsTables, numbers: &Numbers) {
    let t0 = up_dot.ld();
    let t1 = numbers.one.ld();
    let t2 = t0 + t1;
    let t3 = numbers.two.ld();
    let t4 = t2 / t3;
    let t5 = numbers.two.ld();
    let t6 = t4 + t5;
    let t7 = tables.gravity.ld();
    let t8 = t6 * t7;
    let t9 = tables.step_time.ld();
    let t10 = t8 * t9;
    let t11 = tables.wheel_loads[4].ld();
    let t12 = t10 * t11;
    let t13 = tables.weight.ld();
    let t14 = t12 * t13;
    contact.contact_slide_limit.st(t14);
}

/// One straight x87 run as plain Rust over `X`.
fn x87_2(ground_scale: &PlainFloat, car: &Car, numbers: &Numbers) {
    let t0 = car.crash_matrix[5].ld();
    let t1 = numbers.sixty_four.ld();
    let t2 = t0 + t1;
    let t3 = numbers.sixty_four.ld();
    let t4 = t2 / t3;
    ground_scale.st(t4);
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "crash_contact line 103",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[
                            (0x1C5D38, In::F),
                            (0x1C5D3C, In::F),
                            (0x21388C, In::F),
                            (0x213FC8, In::F),
                            (0x214170, In::F),
                            (0x214294, In::F),
                        ],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[(0, In::F)],
                        writes: &[],
                    },
                    Operand::RaceBlock {
                        reads: &[],
                        writes: &[(0x2D3100, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    x87_1(
                        crate::math::x87_mem::view::<PlainFloat>(a[1], 0),
                        crate::test_support::own::race().physics.contact.get(),
                        crate::test_support::own::race().shared.physics_tables.get(),
                        crate::test_support::own::race().course.numbers.get(),
                    );
                },
            },
            Block {
                name: "crash_contact line 120",
                operands: vec![
                    Operand::RaceBlock {
                        reads: &[(0x1C5D60, In::F), (0x31A0A0, In::F)],
                        writes: &[],
                    },
                    Operand::Buffer {
                        reads: &[],
                        writes: &[(0, Out::F)],
                    },
                ],
                ulps: 0,
                plain: |a| {
                    x87_2(
                        crate::math::x87_mem::view::<PlainFloat>(a[1], 0),
                        crate::test_support::own::race().shared.car(),
                        crate::test_support::own::race().course.numbers.get(),
                    );
                },
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("physics.crash_contact", &blocks(), 1200, 100);
    }
}
