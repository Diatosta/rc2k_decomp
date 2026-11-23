//! The force of one contact, into the record the caller passes. From the contact's point
//! (`contact_point`, car frame), the car's speed, its turn speed and the surface's normal: the
//! point's speed (turn x point + speed) goes into the point's speed; its depth along the
//! normal, held at least 1 - |normal y| + `hundredth_a`, into the depth; the push the turn
//! would give along the normal into the turn push and the share `SHARE_NUMERATOR` /
//! (it + 1) into the share; the normal push, the speed along and across the normal, with
//! `contact_step` a part of the normal x `NORMAL_FACTOR` x it / `contact_steps` added;
//! the slide against the motion (the slide) held to `contact_slide_limit` long; their sum
//! (the total) turned by the inverse matrix and divided by `step_time` becomes the record's
//! force, and the point turned becomes its point. The vector steps (cross and dot product, scale,
//! length, turns by the inverse and the car matrix) are the value versions in
//! `physics::vector_bits`.
//!
//! The flow and the calls are Rust; each straight float run is plain Rust over `X` in the
//! `x87_1` .. `x87_8` functions below.
//!
//! Takes: the record the force goes to.

use crate::math::cell_x87::PlainX87;
use crate::math::matrix3::{extended, rounded_value};
use crate::math::x87::{compare_80, neg_rounded};
use crate::math::x87_float::compare;
use crate::math::x87_float::int;
use crate::math::x87_float::{X, f80, ld, st};
use crate::physics::car_store::Car;
use crate::physics::contact_store::ContactWork;
use crate::physics::flow::ZERO_BITS as ZERO;
use crate::physics::tables_store::PhysicsTables;
use crate::physics::tables_store::RecordAt;
use crate::physics::vector_bits::{
    add, cross, dot, hold_low, length, multiply, negate, scale, turn,
};
use crate::race::numbers_store::Numbers;
use std::cell::RefCell;

/// The numerator of the share the turn's push gives (a float as the original's table holds it).
const SHARE_NUMERATOR: f32 = 1.1;
/// The factor on the normal part a contact step adds.
const NORMAL_FACTOR: f32 = 0.1;

/// `record` the one the contact's force goes to. The contact work is borrowed to read the
/// vector and to write the results, not while the force is worked out.
pub(super) fn contact_force(
    contact: &RefCell<ContactWork>,
    numbers: &Numbers,
    physics_tables: &mut PhysicsTables,
    car: &Car,
    record: RecordAt,
) {
    let (turn_speed, point, car_speed, normal, slide_limit) = {
        let work = contact.borrow();
        (
            work.contact_turn_speed,
            work.contact_point,
            work.contact_car_speed,
            work.contact_normal,
            work.contact_slide_limit,
        )
    };
    let turn_cross = cross(turn_speed, point);
    let point_speed = add(turn_cross, car_speed);
    let mut depth = neg_rounded(dot(point_speed, normal)).to_bits();
    depth = hold_low(depth, x87_2(normal[1], numbers));
    // The push the turn would give.
    let lever = turn(&car.car_inverse_matrix, cross(point, normal));
    let inertia = numbers.inertia.map(|c| c.bits());
    let lever_inertia = turn(&car.crash_matrix, multiply(lever, inertia));
    let lever = cross(point, lever_inertia);
    let turn_push = neg_rounded(dot(lever, normal)).to_bits();
    let (_share, factor) = x87_3(ld(SHARE_NUMERATOR.to_bits()), numbers, turn_push, depth);
    // The normal push, the speed along the normal and across it.
    let mut normal_push = scale(normal, extended(factor));
    let along = dot(point_speed, normal);
    let normal_speed = scale(normal, along);
    let side_speed = x87_4(point_speed, normal_speed);
    let speed_length = rounded_value(length(normal_speed)).to_bits();
    let contact_value = rounded_value(length(side_speed)).to_bits();
    let mut extra = [0; 3];
    if physics_tables.contact_step != 0 {
        normal_push = [0; 3];
        extra = normal;
        let part = x87_5(ld(NORMAL_FACTOR.to_bits()), physics_tables);
        extra = scale(extra, part);
    }
    normal_push = add(normal_push, extra);
    let slide_limit_held = crate::math::x87::loaded(slide_limit).to_bits();
    // The slide against the motion, held to the limit's length.
    let mut slide = negate(side_speed);
    if compare_80(length(slide), ZERO) & 0x4000 != 0 {
        slide = [0; 3];
    }
    let (slide_length, status) = x87_6(slide, slide_limit_held);
    if status & 0x4100 == 0 {
        slide = x87_7(slide_limit_held, slide, slide_length).0;
    }
    // The sum into the record.
    let total = turn(&car.car_inverse_matrix, add(slide, normal_push));
    let record = physics_tables.record_mut(record);
    x87_8(total, numbers, &mut record.force);
    let turned_point = turn(&car.car_inverse_matrix, point);
    record.point = turned_point.map(f32::from_bits);
    let mut work = contact.borrow_mut();
    work.contact_speed_length = speed_length;
    work.contact_value = contact_value;
    work.contact_point = turned_point;
}

// ---- the x87 arithmetic ----

/// One straight x87 run in plain Rust over `X`: the contact's normal y gives the lowest depth,
/// `1 - |y| + hundredth`, stored as a float's bits.
fn x87_2(normal_y: u32, numbers: &Numbers) -> u32 {
    let t0 = X(1.0);
    let t1 = ld(normal_y);
    let t2 = t1.abs();
    let t3 = t0 - t2;
    let t4 = numbers.hundredth_a.ld();
    let t5 = t3 + t4;
    st(t5)
}

/// The share and the factor on the normal push, as plain Rust over `X`:
/// `share = numerator / (turn_push + one)`, `factor = depth * share`; both as float bits.
fn x87_3(numerator: X, numbers: &Numbers, turn_push: u32, depth: u32) -> (u32, u32) {
    let t1 = ld(turn_push);
    let t2 = numbers.one.ld();
    let t3 = t1 + t2;
    let t4 = numerator / t3;
    let share = st(t4);
    let t5 = ld(depth);
    let t6 = ld(share);
    let t7 = t5 * t6;
    (share, st(t7))
}

/// One straight x87 run as plain Rust over `X`: the speed across the normal is the point's speed
/// less the speed along it.
fn x87_4(side_speed: [u32; 3], normal_speed: [u32; 3]) -> [u32; 3] {
    std::array::from_fn(|k| st(ld(side_speed[k]) - ld(normal_speed[k])))
}

/// The part of the normal a contact step adds, `(factor * contact_step) / contact_steps`, as 80
/// bits.
fn x87_5(normal_factor: X, tables: &PhysicsTables) -> [u8; 10] {
    let t1 = int(tables.contact_step as i32);
    let t2 = normal_factor * t1;
    let t3 = int(tables.contact_steps as i32);
    let t4 = t2 / t3;
    f80(t4)
}

/// One straight x87 run as plain Rust over `X`: the slide's length (stored as the work value),
/// compared with the limit.
fn x87_6(slide: [u32; 3], slide_limit_held: u32) -> (u32, u16) {
    let t0 = ld(slide[0]);
    let t1 = t0 * t0;
    let t2 = ld(slide[1]);
    let t3 = t2 * t2;
    let t4 = ld(slide[2]);
    let t5 = t4 * t4;
    let t6 = t3 + t5;
    let t7 = t1 + t6;
    let t8 = t7.sqrt();
    let t9 = ld(slide_limit_held);
    (st(t8), compare(t8, t9))
}

/// One straight x87 run as plain Rust over `X`: the slide scaled to the limit's length. Returns
/// the slide and the work value it leaves.
fn x87_7(slide_limit_held: u32, slide: [u32; 3], scratch: u32) -> ([u32; 3], u32) {
    let t0 = ld(slide_limit_held);
    let t1 = ld(scratch);
    let t2 = t0 / t1;
    (slide.map(|part| st(ld(part) * t2)), st(t2))
}

/// One straight x87 run as plain Rust over `X`: the total over the step time.
fn x87_8(total: [u32; 3], numbers: &Numbers, d: &mut [f32; 3]) {
    for k in 0..3 {
        let t2 = ld(total[k]) / numbers.step_time.ld();
        d[k].st(t2);
    }
}

#[cfg(test)]
mod tests;
