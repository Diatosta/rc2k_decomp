//! Whether a point of an object lies inside the work car's box. The
//! point (three whole numbers) plus the object's place plus `object_offset` less the car's
//! `position` goes into `object_point_offset`; that divided by 65536 (`k65536`) goes into
//! `object_point_scaled` and `object_point_turned`, is turned by the car's heading
//! (`physics::turn_by_heading`) and back to whole numbers (times 65536, rounded) into
//! `contact_point`. Inside when x is below `spec_ints[2]` and not below minus it, and z is below
//! `spec_ints[0]` and not below minus `spec_ints[1]` (each by the sign of the comparison). Inside
//! with the contact kind (`contact_kind`) 10, 1, 5 or 6 also: the point's x, 0, z (/ 65536) go into
//! the car's `contact_point`, the car's speed length (`velocity`, `physics::vector_length`) into
//! `contact_speed_length`, `contact_depth` is raised to its size, `contact_flag` = 1 and
//! `wall_hit_timer` = 0x32.

use crate::errors::Refused;
use crate::math::matrix3::rounded_value;
use crate::math::x87_float::X;
use crate::math::x87_float::int;
use crate::math::x87_float::{ld, st, to_int};
use crate::physics::car_store::Car;
use crate::physics::contact_store::ContactWork;
use crate::physics::step_families::ContactInputs;
use crate::physics::turn_by_heading::turn_by_heading;
use crate::physics::vector_length::vector_length;
use crate::race::numbers_store::Numbers;
use rc2k_core::physics::contacts;

/// Whether the point `point` (its three whole numbers, read by the caller) of the object of course
/// face `face` (its number) is inside.
// Core: `rc2k_core::physics::contacts::{point_part, inside_box, kind_hits}` (the x87 steps stay).
pub(super) fn point_hits_car(
    c: &ContactInputs<'_>,
    point: [u32; 3],
    face: u32,
) -> Result<bool, Refused> {
    let rd =
        |offset: u32| crate::race::course_places::face_field::<u32>(c.course.reads(), face, offset);
    let mut y = 0;
    for k in 0..3u32 {
        let offset = c.contact.borrow().object_offset[k as usize];
        let v = contacts::point_part(
            point[k as usize],
            rd(0x30 + 4 * k)?,
            offset,
            c.car.position.bits_at(k as usize),
        );
        c.contact.borrow_mut().object_point_offset[k as usize] = v;
        if k == 1 {
            y = v;
        }
    }
    let scaled = {
        let mut work = c.contact.borrow_mut();
        x87_1(&mut work, c.numbers);
        work.object_point_turned = work.object_point_scaled;
        work.object_point_turned
    };
    let turned = turn_by_heading(c.shared.buffers.engine_tables(), c.car, scaled, y);
    let (x, z, kind) = {
        let mut work = c.contact.borrow_mut();
        work.object_point_turned = turned;
        x87_2(&mut work, c.numbers);
        (
            work.contact_point[0],
            work.contact_point[2],
            work.contact_kind,
        )
    };
    if !contacts::inside_box(
        x,
        z,
        c.tables.spec_ints[2].get(),
        c.tables.spec_ints[0].get(),
        c.tables.spec_ints[1].get(),
    ) {
        return Ok(false);
    }
    if contacts::kind_hits(kind) {
        x87_3(&c.contact.borrow(), c.numbers, c.car);
        let speed_length = rounded_value(vector_length(&c.car.velocity));
        c.contact.borrow_mut().contact_speed_length = speed_length.to_bits();
        if let Some(hit) =
            rc2k_core::physics::walls::raised_hit(speed_length, c.car.contact_depth.get())
        {
            c.car.contact_depth.set(hit);
        }
        c.car.contact_flag.set(1);
        c.car.wall_hit_timer.set(0x32);
    }
    Ok(true)
}

// ---- the x87 arithmetic ----

/// One straight x87 run as plain Rust over `X`.
fn x87_1(contact: &mut ContactWork, numbers: &Numbers) {
    for k in 0..3 {
        let t0 = int(contact.object_point_offset[k] as i32);
        let t1 = numbers.k65536.ld();
        let t2 = t0 / t1;
        contact.object_point_scaled[k] = st(t2);
    }
}

/// One straight x87 run as plain Rust over `X`: the turned point times 65536 as whole numbers.
fn x87_2(contact: &mut ContactWork, numbers: &Numbers) {
    for k in 0..3 {
        let t0 = ld(contact.object_point_turned[k]);
        let t1 = numbers.k65536.ld();
        let t2 = t0 * t1;
        contact.contact_point[k] = to_int(t2) as u32;
    }
}

/// One straight x87 run as plain Rust over `X`.
fn x87_3(contact: &ContactWork, numbers: &Numbers, car: &Car) {
    let t0 = int(contact.contact_point[0] as i32);
    let t1 = numbers.k65536.ld();
    let t2 = t0 / t1;
    car.contact_point.st_at(0, t2);
    let t3 = X(0.0);
    car.contact_point.st_at(1, t3);
    let t4 = int(contact.contact_point[2] as i32);
    let t5 = numbers.k65536.ld();
    let t6 = t4 / t5;
    car.contact_point.st_at(2, t6);
}

#[cfg(test)]
mod tests;
