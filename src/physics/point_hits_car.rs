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
use crate::math::cell_x87::CellX87;
use crate::math::matrix3::rounded_value;
use crate::math::x87_float::X;
use crate::math::x87_float::int;
use crate::math::x87_float::to_int;
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
    let contact = c.contact;
    let rd =
        |offset: u32| crate::race::course_places::face_field::<u32>(c.course.reads(), face, offset);
    let mut y = 0;
    for k in 0..3u32 {
        let v = contacts::point_part(
            point[k as usize],
            rd(0x30 + 4 * k)?,
            contact.object_offset[k as usize].get(),
            c.car.position[k as usize].get(),
        );
        contact.object_point_offset[k as usize].set(v);
        if k == 1 {
            y = v;
        }
    }
    x87_1(c.contact, c.numbers);
    for k in 0..3 {
        contact.object_point_turned[k].set(contact.object_point_scaled[k].get());
    }
    turn_by_heading(
        &c.shared.buffers.reads.get().engine_tables,
        c.car,
        &contact.object_point_turned,
        y,
    );
    x87_2(c.contact, c.numbers);
    let (x, z) = (
        contact.contact_point[0].bits(),
        contact.contact_point[2].bits(),
    );
    if !contacts::inside_box(
        x,
        z,
        c.tables.spec_ints[2].get(),
        c.tables.spec_ints[0].get(),
        c.tables.spec_ints[1].get(),
    ) {
        return Ok(false);
    }
    if contacts::kind_hits(contact.contact_kind.get()) {
        x87_3(c.contact, c.numbers, c.car);
        contact
            .contact_speed_length
            .set(rounded_value(vector_length(&c.car.velocity)));
        if let Some(hit) = rc2k_core::physics::walls::raised_hit(
            contact.contact_speed_length.get(),
            c.car.contact_depth.get(),
        ) {
            c.car.contact_depth.set(hit);
        }
        c.car.contact_flag.set(1);
        c.car.wall_hit_timer.set(0x32);
    }
    Ok(true)
}

// ---- the x87 arithmetic ----

/// One straight x87 run as plain Rust over `X`.
fn x87_1(contact: &ContactWork, numbers: &Numbers) {
    let t0 = int(contact.object_point_offset[0].get() as i32);
    let t1 = numbers.k65536.ld();
    let t2 = t0 / t1;
    contact.object_point_scaled[0].st(t2);
    let t3 = int(contact.object_point_offset[1].get() as i32);
    let t4 = numbers.k65536.ld();
    let t5 = t3 / t4;
    contact.object_point_scaled[1].st(t5);
    let t6 = int(contact.object_point_offset[2].get() as i32);
    let t7 = numbers.k65536.ld();
    let t8 = t6 / t7;
    contact.object_point_scaled[2].st(t8);
}

/// One straight x87 run as plain Rust over `X`.
fn x87_2(contact: &ContactWork, numbers: &Numbers) {
    let t0 = contact.object_point_turned[0].ld();
    let t1 = numbers.k65536.ld();
    let t2 = t0 * t1;
    crate::physics::flow::set_whole(&contact.contact_point[0], to_int(t2));
    let t3 = contact.object_point_turned[1].ld();
    let t4 = numbers.k65536.ld();
    let t5 = t3 * t4;
    crate::physics::flow::set_whole(&contact.contact_point[1], to_int(t5));
    let t6 = contact.object_point_turned[2].ld();
    let t7 = numbers.k65536.ld();
    let t8 = t6 * t7;
    crate::physics::flow::set_whole(&contact.contact_point[2], to_int(t8));
}

/// One straight x87 run as plain Rust over `X`.
fn x87_3(contact: &ContactWork, numbers: &Numbers, car: &Car) {
    let t0 = int(crate::physics::flow::whole(&contact.contact_point[0]));
    let t1 = numbers.k65536.ld();
    let t2 = t0 / t1;
    car.contact_point[0].st(t2);
    let t3 = X(0.0);
    car.contact_point[1].st(t3);
    let t4 = int(crate::physics::flow::whole(&contact.contact_point[2]));
    let t5 = numbers.k65536.ld();
    let t6 = t4 / t5;
    car.contact_point[2].st(t6);
}

#[cfg(test)]
mod tests;
