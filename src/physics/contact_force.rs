//! The force of one contact, into the record `contact_record` names. From the contact's point
//! (`contact_point`, car frame), the car's speed, its turn speed and the surface's normal: the
//! point's speed (turn x point + speed) goes into the point's speed; its depth along the
//! normal, held at least 1 - |normal y| + `hundredth_a`, into the depth; the push the turn
//! would give along the normal into the turn push and the share `SHARE_NUMERATOR` /
//! (it + 1) into the share; the normal push, the speed along and across the normal, with
//! `contact_step` a part of the normal x `NORMAL_FACTOR` x it / `contact_steps` added;
//! the slide against the motion (the slide) held to `contact_slide_limit` long; their sum
//! (the total) turned by the inverse matrix and divided by `step_time` becomes the record's
//! force, and the point turned becomes its point. The helpers are `physics::turn_by_inverse`,
//! `turn_by_car_matrix`, `scene::cross_product`, `physics::dot_product`, `multiply_parts`,
//! `scale_vector` and `vector_length`.
//!
//! The flow and the calls are Rust; each straight float run is plain Rust over `X` in the
//! `x87_1` .. `x87_8` functions below.
//!
//! Takes: the record the force goes to (`contact_record`, set by each caller just before).

use crate::math::cell_x87::CellX87;
use crate::math::matrix3::{extended, rounded_value};
use crate::math::x87::{compare_80, neg_rounded};
use crate::math::x87_float::compare;
use crate::math::x87_float::int;
use crate::math::x87_float::{X, f80};
use crate::memory::session::{FloatLike, PlainFloat};
use crate::physics::car_store::Car;
use crate::physics::contact_store::ContactWork;
use crate::physics::dot_product::dot_product;
use crate::physics::flow::{ZERO_BITS as ZERO, hold_low, negate, p};
use crate::physics::multiply_parts::multiply_parts;
use crate::physics::scale_vector::scale_vector;
use crate::physics::tables_store::ForceRecord;
use crate::physics::tables_store::PhysicsTables;
use crate::physics::turn_by_car_matrix::turn_by_car_matrix;
use crate::physics::turn_by_inverse::turn_by_inverse;
use crate::physics::vector_length::vector_length;
use crate::race::numbers_store::Numbers;
use crate::scene::view_matrix::cross_product;

/// The numerator of the share the turn's push gives (a float as the original's table holds it).
const SHARE_NUMERATOR: f32 = 1.1;
/// The factor on the normal part a contact step adds.
const NORMAL_FACTOR: f32 = 0.1;

/// The work of one contact's force: every value is written before it is read within
/// `contact_force`, so one call's work never reaches the next.
struct ForceWork {
    turn_cross: [PlainFloat; 3],
    point_speed: [PlainFloat; 3],
    normal_push: [PlainFloat; 3],
    side_speed: [PlainFloat; 3],
    normal_speed: [PlainFloat; 3],
    slide: [PlainFloat; 3],
    total: [PlainFloat; 3],
    extra: [PlainFloat; 3],
    lever: [PlainFloat; 3],
    lever_inertia: [PlainFloat; 3],
    depth: PlainFloat,
    turn_push: PlainFloat,
    share: PlainFloat,
    factor: PlainFloat,
    slide_limit_held: PlainFloat,
}

impl ForceWork {
    fn new() -> Self {
        Self {
            turn_cross: PlainFloat::array([0.0; 3]),
            point_speed: PlainFloat::array([0.0; 3]),
            normal_push: PlainFloat::array([0.0; 3]),
            side_speed: PlainFloat::array([0.0; 3]),
            normal_speed: PlainFloat::array([0.0; 3]),
            slide: PlainFloat::array([0.0; 3]),
            total: PlainFloat::array([0.0; 3]),
            extra: PlainFloat::array([0.0; 3]),
            lever: PlainFloat::array([0.0; 3]),
            lever_inertia: PlainFloat::array([0.0; 3]),
            depth: PlainFloat::new(0.0),
            turn_push: PlainFloat::new(0.0),
            share: PlainFloat::new(0.0),
            factor: PlainFloat::new(0.0),
            slide_limit_held: PlainFloat::new(0.0),
        }
    }
}

/// Three floats copied as bits, in order.
fn copy3(to: &[impl FloatLike], from: &[impl FloatLike]) {
    for k in 0..3 {
        to[k].set(from[k].get());
    }
}

/// Three floats set to the bits `v`.
fn fill3(to: &[impl FloatLike], v: u32) {
    for cell in &to[..3] {
        cell.set_bits(v);
    }
}

/// `record` the one the contact's force goes to (`contact_record`, set by each caller
/// just before).
pub(super) fn contact_force(
    contact: &ContactWork,
    numbers: &Numbers,
    physics_tables: &PhysicsTables,
    car: &Car,
    record: &ForceRecord,
) {
    // The work value the first and the sliding steps pass through; each is stored before it is read.
    let scratch = crate::memory::session::PlainDword::new(0);
    let w = ForceWork::new();
    cross_product(
        &contact.contact_turn_speed,
        &contact.contact_point,
        &w.turn_cross,
    );
    x87_1(&w.turn_cross, &contact.contact_car_speed, &w.point_speed);
    p(
        &w.depth,
        neg_rounded(dot_product(&w.point_speed, &contact.contact_normal)).to_bits(),
    );
    x87_2(contact, numbers, &scratch);
    hold_low(&w.depth, &scratch);
    // The push the turn would give.
    cross_product(&contact.contact_point, &contact.contact_normal, &w.lever);
    turn_by_inverse(car, &w.lever);
    multiply_parts(&w.lever, &physics_tables.inertia, &w.lever_inertia);
    turn_by_car_matrix(car, &w.lever_inertia);
    cross_product(&contact.contact_point, &w.lever_inertia, &w.lever);
    p(
        &w.turn_push,
        neg_rounded(dot_product(&w.lever, &contact.contact_normal)).to_bits(),
    );
    x87_3(
        crate::math::x87_float::ld(SHARE_NUMERATOR.to_bits()),
        numbers,
        &w.turn_push,
        &w.depth,
        &w.share,
        &w.factor,
    );
    // The normal push, the speed along the normal and across it.
    copy3(&w.normal_push, &contact.contact_normal);
    let factor = extended(w.factor.get());
    scale_vector(&w.normal_push, factor);
    let along = { dot_product(&w.point_speed, &contact.contact_normal) };
    copy3(&w.normal_speed, &contact.contact_normal);
    scale_vector(&w.normal_speed, along);
    copy3(&w.side_speed, &w.point_speed);
    x87_4(&w.side_speed, &w.normal_speed);
    contact
        .contact_speed_length
        .set(rounded_value(vector_length(&w.normal_speed)));
    contact
        .contact_value
        .set(rounded_value(vector_length(&w.side_speed)));
    fill3(&w.extra, 0);
    if physics_tables.contact_step.get() != 0 {
        fill3(&w.normal_push, 0);
        copy3(&w.extra, &contact.contact_normal);
        let part = x87_5(
            crate::math::x87_float::ld(NORMAL_FACTOR.to_bits()),
            physics_tables,
        );
        scale_vector(&w.extra, part);
    }
    x87_1(&w.normal_push, &w.extra, &w.normal_push);
    copy3(&w.total, &w.normal_push);
    w.slide_limit_held
        .set(crate::math::x87::loaded(&contact.contact_slide_limit));
    // The slide against the motion, held to the limit's length.
    copy3(&w.slide, &w.side_speed);
    negate(&w.slide[0]);
    negate(&w.slide[1]);
    negate(&w.slide[2]);
    if compare_80(vector_length(&w.slide), ZERO) & 0x4000 != 0 {
        fill3(&w.slide, 0);
    }
    let status = x87_6(&w.slide, &w.slide_limit_held, &scratch);
    if status & 0x4100 == 0 {
        x87_7(&w.slide_limit_held, &w.slide, &scratch);
    }
    // The sum into the record.
    x87_1(&w.slide, &w.normal_push, &w.total);
    turn_by_inverse(car, &w.total);
    x87_8(&w.total, physics_tables, &record.force);
    turn_by_inverse(car, &contact.contact_point);
    for k in 0..3 {
        record.point[k].set(contact.contact_point[k].get());
    }
}

// ---- the x87 arithmetic ----

/// An x87 run in plain Rust over `X`: three floats added, `out[k] = a[k] + b[k]` (each stored as
/// made, so `out` may be one of the two).
fn x87_1(a: &[PlainFloat; 3], b: &[PlainFloat; 3], out: &[PlainFloat; 3]) {
    for k in 0..3 {
        out[k].st(a[k].ld() + b[k].ld());
    }
}

/// One straight x87 run as plain Rust over `X`.
fn x87_2(contact: &ContactWork, numbers: &Numbers, scratch: &crate::memory::session::PlainDword) {
    let t0 = X(1.0);
    let t1 = contact.contact_normal[1].ld();
    let t2 = t1.abs();
    let t3 = t0 - t2;
    let t4 = numbers.hundredth_a.ld();
    let t5 = t3 + t4;
    scratch.st(t5);
}

/// The share and the factor on the normal push, as plain Rust over `X`:
/// `share = numerator / (turn_push + one)`, `factor = depth * share`.
fn x87_3(
    numerator: X,
    numbers: &Numbers,
    turn_push: &PlainFloat,
    depth: &PlainFloat,
    share: &PlainFloat,
    factor: &PlainFloat,
) {
    let t1 = turn_push.ld();
    let t2 = numbers.one.ld();
    let t3 = t1 + t2;
    let t4 = numerator / t3;
    share.st(t4);
    let t5 = depth.ld();
    let t6 = share.ld();
    let t7 = t5 * t6;
    factor.st(t7);
}

/// One straight x87 run as plain Rust over `X`: the speed across the normal is the point's speed
/// less the speed along it.
fn x87_4(side_speed: &[PlainFloat; 3], normal_speed: &[PlainFloat; 3]) {
    for k in 0..3 {
        let t2 = side_speed[k].ld() - normal_speed[k].ld();
        side_speed[k].st(t2);
    }
}

/// The part of the normal a contact step adds, `(factor * contact_step) / contact_steps`, as 80
/// bits.
fn x87_5(normal_factor: X, tables: &PhysicsTables) -> [u8; 10] {
    let t1 = int(tables.contact_step.get() as i32);
    let t2 = normal_factor * t1;
    let t3 = int(tables.contact_steps.get() as i32);
    let t4 = t2 / t3;
    f80(t4)
}

/// One straight x87 run as plain Rust over `X`: the slide's length into `scratch`, compared with
/// the limit.
fn x87_6(
    slide: &[PlainFloat; 3],
    slide_limit_held: &PlainFloat,
    scratch: &crate::memory::session::PlainDword,
) -> u16 {
    let t0 = slide[0].ld();
    let t1 = t0 * t0;
    let t2 = slide[1].ld();
    let t3 = t2 * t2;
    let t4 = slide[2].ld();
    let t5 = t4 * t4;
    let t6 = t3 + t5;
    let t7 = t1 + t6;
    let t8 = t7.sqrt();
    scratch.st(t8);
    let t9 = slide_limit_held.ld();
    compare(t8, t9)
}

/// One straight x87 run as plain Rust over `X`: the slide scaled to the limit's length.
fn x87_7(
    slide_limit_held: &PlainFloat,
    slide: &[PlainFloat; 3],
    scratch: &crate::memory::session::PlainDword,
) {
    let t0 = slide_limit_held.ld();
    let t1 = scratch.ld();
    let t2 = t0 / t1;
    for cell in slide {
        let t4 = cell.ld() * t2;
        cell.st(t4);
    }
    scratch.st(t2);
}

/// One straight x87 run as plain Rust over `X`: the total over the step time.
fn x87_8(total: &[PlainFloat; 3], tables: &PhysicsTables, d: &[PlainFloat; 3]) {
    for k in 0..3 {
        let t2 = total[k].ld() / tables.step_time.ld();
        d[k].st(t2);
    }
}

#[cfg(test)]
mod tests;
