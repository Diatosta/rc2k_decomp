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
use crate::math::x87_float::st;
use crate::memory::session::PlainFloat;
use crate::physics::car_store::Car;
use crate::physics::contact_force::contact_force;
use crate::physics::contact_store::{ContactWork, ProbeLists};
use crate::physics::flow::FloatView;
use crate::physics::flow::hold_high;
use crate::physics::lowest_body_point::lowest_body_point;
use crate::physics::place_sound_e5::place_sound_e5;
use crate::physics::sound_cue_e5::sound_cue_e5;
use crate::physics::step_families::ContactInputs;
use crate::physics::tables_store::PhysicsTables;
use crate::physics::vector_bits::{bits_of, dot};
use crate::race::numbers_store::Numbers;
use crate::race::owner_course::Course;
use crate::scene::rotate_point::rotate_point;
use rc2k_core::physics::contacts;
use std::cell::RefCell;

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
    contact: &RefCell<ContactWork>,
    probes: &ProbeLists,
    course: &Course,
    car: &Car,
    n: usize,
) -> Result<(), Refused> {
    let face = car.probe_point_face[n].get();
    let parts = contacts::normal_offsets(probes.probe_face_flip[n]);
    let mut normal = [0; 3];
    for (k, &o) in parts.iter().enumerate() {
        let v = crate::race::course_places::face_field::<u32>(course.reads(), face, o)?;
        normal[k] = loaded(v).to_bits();
    }
    let mut work = contact.borrow_mut();
    work.contact_car_speed = bits_of(&car.velocity);
    work.contact_normal = normal;
    work.contact_turn_speed = bits_of(&car.spin);
    work.contact_point = [
        probes.probe_turned_x[n].to_bits(),
        probes.probe_turned_y[n].to_bits(),
        probes.probe_turned_z[n].to_bits(),
    ];
    Ok(())
}

/// The deepest point's number (`lowest_point`) for the long path; refused past 0x3F.
pub fn point_number(contact: &RefCell<ContactWork>) -> Result<usize, Refused> {
    let n = contact.borrow().lowest_point;
    contacts::check_body_point(n)?;
    Ok(n as usize)
}

// Core: `rc2k_core::physics::contacts::{touches, normal_offsets, check_body_point}`
// (the x87 steps stay).
fn steps(c: &ContactInputs<'_>) -> Result<(), Refused> {
    c.car.contact_found.set(0);
    { lowest_body_point(c.car, c.numbers, c.contact, &c.ground.probes.borrow()) }?;
    let depth = c.contact.borrow().lowest_depth;
    if !contacts::touches(
        depth,
        crate::physics::figures::CRASH_DEPTH,
        c.car.all_wheels_off.get() != 0,
    ) {
        { sound_cue_e5(c.sound.frame_work, c.work, c.sound.owner) }?;
        return Ok(());
    }
    let n = point_number(c.contact)?;
    {
        let probes = c.ground.probes.borrow();
        c.car.contact_point_x.set(probes.probe_x[n]);
        c.car
            .contact_point_ground
            .set(probes.probe_ground_height[n]);
        c.car.contact_point_z.set(probes.probe_z[n]);
        c.car.contact_found.set(1);
        take_contact(c.contact, &probes, c.course, c.car, n)?;
    }
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
    let normal = c.contact.borrow().contact_normal;
    up_dot.set_float(neg_rounded(dot(bits_of(&car_up), normal)));
    let slide_limit = x87_1(&up_dot, c.tables, c.numbers);
    c.contact.borrow_mut().contact_slide_limit = slide_limit;
    contact_force(c.contact, c.numbers, c.tables, c.car, &c.tables.forces[21]);
    { place_sound_e5(&c.sound) }?;
    for cell in &c.tables.forces[21].point {
        cell.set(0.0);
    }
    x87_2(&ground_scale, c.car, c.numbers);
    hold_high(&ground_scale, &c.numbers.one);
    c.car.spin[1].set(mul(&ground_scale, &c.car.spin[1]));
    let contact_value = c.contact.borrow().contact_value;
    c.car.contact_value.set_bits(contact_value);
    Ok(())
}

pub fn crash_contact(c: &ContactInputs<'_>) -> Result<(), Refused> {
    steps(c)
}

// ---- the x87 arithmetic ----

/// The game's x87 arithmetic as plain Rust over `X`, on the variables
/// named below (`[name]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// A long formula, covered by the tests.
fn x87_1(up_dot: &PlainFloat, tables: &PhysicsTables, numbers: &Numbers) -> u32 {
    let t0 = up_dot.ld();
    let t1 = numbers.one.ld();
    let t2 = t0 + t1;
    let t3 = numbers.two.ld();
    let t4 = t2 / t3;
    let t5 = numbers.two.ld();
    let t6 = t4 + t5;
    let t7 = tables.gravity.ld();
    let t8 = t6 * t7;
    let t9 = numbers.step_time.ld();
    let t10 = t8 * t9;
    let t11 = tables.wheel_loads[4].ld();
    let t12 = t10 * t11;
    let t13 = tables.weight.ld();
    let t14 = t12 * t13;
    st(t14)
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
                    let limit = x87_1(
                        crate::math::x87_mem::view::<PlainFloat>(a[1], 0),
                        crate::test_support::own::race().shared.physics_tables.get(),
                        &crate::test_support::race_places::numbers(),
                    );
                    crate::test_support::own::race()
                        .physics
                        .contact()
                        .borrow_mut()
                        .contact_slide_limit = limit;
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
                        &crate::test_support::race_places::numbers(),
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
