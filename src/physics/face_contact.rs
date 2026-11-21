//! Tries one course face against the work car. `face_number` = n. Faces hidden by kind
//! (`scene::kind_hidden_at`) and faces whose first byte is not 0xFF (not an object's) are left.
//! An object byte of 0x3B: the kind 0xFF, `physics::touch_face`. Otherwise the object
//! (`face_object`) and its contact kind (`contact_kind`) come from its material record; the
//! face is left when the record's flag bit 0 is set, when its direction bits are not 0 and not
//! the stage direction + 1, or when the object is 0x38 or more but for 0x3A. Its turn and scale
//! are set up and `physics::scene_point_hits_car` runs. Not inside: for kinds 9, 8, 6, 5, when
//! the point's distance from the car (x, z) is within the object's reach (`radii[n]` x the
//! scale >> 4, + 0x28000), `physics::object_hits_car` runs. Inside either way: the contact's
//! angle (`face_angle`: the arctangent, + 0x80 & 0xFF), the car's speed x 65536 (`face_speed`)
//! and the point / 65536 (`object_point_turned`); then by kind: 2, 7, 3, 8, 4 `touch_face`; 6,
//! 9, 5, 1, 10 `physics::restore_saved_car`, `physics::feel_ground`, `touch_face` and the push
//! back: minus the car's speed, level (y 0), made one long (`race::normalise_work_vector`) as
//! the normal, with the speeds and the point (y 0) taken, no slide limit, and
//! `physics::contact_force` into contact record 0.
//!
//! Refuses a kind past the 0x4B0 material records (but for an unloaded slot's,
//! `contacts::UNSET_KIND`: stage 22 has such a face), and what its calls refuse.

use crate::errors::Refused;
use crate::math::x87::compare_80;
use crate::math::x87::int_hypot;
use crate::math::x87::length_80;
use crate::math::x87_float::to_int;
use crate::math::x87_float::{int, st};
use crate::physics::car_store::Car;
use crate::physics::contact_force::contact_force;
use crate::physics::contact_store::ContactWork;
use crate::physics::flow::{ZERO_BITS as ZERO, negated_velocity};
use crate::physics::step_families::ContactInputs;
use crate::physics::touch_face::touch_face;
use crate::physics::vector_bits::bits_of;
use crate::race::norms::normalise_work_vector;
use crate::race::numbers_store::Numbers;
use rc2k_core::physics::face_contacts::{self as contacts, FaceObject, Inside};

fn touch(c: &ContactInputs<'_>, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    { touch_face(c, scene) }?;
    Ok(())
}

/// The steps for face `number` .
// Core: `rc2k_core::physics::face_contacts` (the x87 steps stay).
pub fn steps(
    c: &ContactInputs<'_>,
    number: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    c.contact.borrow_mut().face_number = number;
    let face = number;
    let face_byte = |offset| -> Result<u32, Refused> {
        Ok(u32::from(crate::race::course_places::face_field::<u8>(
            c.course.reads(),
            face,
            offset,
        )?))
    };
    let face_dword = |offset| -> Result<u32, Refused> {
        Ok(crate::race::course_places::face_field::<u32>(
            c.course.reads(),
            face,
            offset,
        )?)
    };
    if crate::scene::kind_hidden_at::kind_hidden_at(c.course, c.shared, face, scene)? {
        return Ok(());
    }
    let mut refused = None;
    let found = contacts::face_object(face_byte(0)?, || {
        face_byte(1).unwrap_or_else(|e| {
            refused = Some(e);
            0
        })
    });
    if let Some(e) = refused {
        return Err(e);
    }
    let object = match found {
        FaceObject::NotObject => return Ok(()),
        FaceObject::TouchOnly => {
            c.contact.borrow_mut().contact_kind = 0xFF;
            return touch(c, scene);
        }
        FaceObject::Object(object) => object,
    };
    c.contact.borrow_mut().face_object = object;
    let kind = scene.objects.tables.borrow().object_kind(object);
    if let Err(e) = contacts::check_material(kind) {
        crate::program_log::log(&format!(
            "physics::face_contact: face {}: object {object}, kind {kind:#x}, car {} at {:?}",
            face,
            c.work.current_car.get(),
            c.car.position.bits().map(|c| c as i32),
        ));
        return Err(e.into());
    }
    // An unset kind wraps to the record before the first.
    let material_kind: u32 =
        crate::scene::polygon::material_run_byte(c.shared.buffers.polygon_tables(), kind, 0x1C)?
            .into();
    c.contact.borrow_mut().contact_kind = material_kind;
    if contacts::object_tried(face_byte(0x25)?, c.work.stage_direction.get(), object).is_err() {
        return Ok(());
    }
    for (k, o) in [8u32, 0xC, 0x10].iter().enumerate() {
        scene.object_work_mut().angles[k] = contacts::turn_word(face_byte(*o)?);
    }
    scene.object_work_mut().size = face_byte(0x18)?;
    // The face is both the record and the object here.
    let (inside, point) = { crate::physics::scene_point_hits_car::scene_point_hits_car(c, face) }?;
    if !inside {
        let contact_kind = c.contact.borrow().contact_kind;
        if !contacts::tries_points(contact_kind) {
            return Ok(());
        }
        let dx = crate::race::scene_points::point_word(
            c.course.reads(),
            &crate::race::scene_points::no_own(),
            point,
            0,
        )?
        .wrapping_add(face_dword(0x30)?)
        .wrapping_sub(c.car.position.bits_at(0));
        let dz = crate::race::scene_points::point_word(
            c.course.reads(),
            &crate::race::scene_points::no_own(),
            point,
            2,
        )?
        .wrapping_add(face_dword(0x38)?)
        .wrapping_sub(c.car.position.bits_at(2));
        let distance = int_hypot(dx as i32, dz as i32) as u32;
        let face_object = c.contact.borrow().face_object;
        let (reach, _) = contacts::reach(
            scene.loaded_slots().radii[face_object as usize],
            scene.object_work().size,
        );
        if contacts::out_of_reach(reach, distance) {
            return Ok(());
        }
        // The scene point is the record, the face the object.
        if !{ crate::physics::object_hits_car::object_hits_car(c, point, face, scene) }? {
            return Ok(());
        }
    }
    // Inside.
    let contact_kind = {
        let mut work = c.contact.borrow_mut();
        x87_1(&mut work, c.numbers);
        work.face_angle = work.face_angle.wrapping_add(0x80) & 0xFF;
        x87_2(&mut work, c.car, c.numbers);
        work.contact_kind
    };
    match contacts::inside(contact_kind) {
        Inside::PushBack => {}
        Inside::Touch => return touch(c, scene),
        Inside::Nothing => return Ok(()),
    }
    {
        crate::physics::restore_saved_car::restore_saved_car(
            c.car,
            c.tables,
            &c.ground.probes.borrow(),
        )
    }?;
    { crate::physics::feel_ground::feel_ground(&c.ground) }?;
    touch(c, scene)?;
    // Pushed back against the motion, level.
    let mut scratch = c.frame.scratch_mut();
    scratch.work_vector = negated_velocity(c.car);
    if compare_80(length_80(scratch.work_vector), ZERO) & 0x4000 != 0 {
        scratch.work_vector = [0.0f32.to_bits(); 3];
    }
    scratch.work_vector[1] = 0;
    normalise_work_vector(c.numbers, &mut scratch, scene);
    let normal = scratch.work_vector;
    drop(scratch);
    {
        let mut work = c.contact.borrow_mut();
        work.contact_normal = normal;
        work.contact_car_speed = bits_of(&c.car.velocity);
        work.contact_turn_speed = bits_of(&c.car.spin);
        work.contact_point = work.object_point_scaled;
        work.contact_point[1] = 0;
        work.contact_slide_limit = 0;
    }
    contact_force(c.contact, c.numbers, c.tables, c.car, &c.tables.contacts[0]);
    Ok(())
}

// ---- the x87 arithmetic ----

/// One straight x87 run as plain Rust over `X`.
fn x87_1(contact: &mut ContactWork, numbers: &Numbers) {
    let t0 = int(contact.object_point_offset[0] as i32);
    let t1 = int(contact.object_point_offset[2] as i32);
    let t2 = t0.atan2(t1);
    let t3 = numbers.k256.ld();
    let t4 = t2 * t3;
    let t5 = numbers.one_over_two_pi.ld();
    let t6 = t4 * t5;
    contact.face_angle = to_int(t6) as u32;
}

/// One straight x87 run as plain Rust over `X`.
fn x87_2(contact: &mut ContactWork, car: &Car, numbers: &Numbers) {
    let t0 = car.velocity.ld(0);
    let t1 = t0 * t0;
    let t2 = car.velocity.ld(2);
    let t3 = t2 * t2;
    let t4 = car.velocity.ld(1);
    let t5 = t4 * t4;
    let t6 = t3 + t5;
    let t7 = t1 + t6;
    let t8 = t7.sqrt();
    let t9 = numbers.k65536.ld();
    let t10 = t8 * t9;
    contact.face_speed = to_int(t10) as u32;
    let t11 = int(contact.contact_point[0] as i32);
    let t12 = numbers.k65536.ld();
    let t13 = t11 / t12;
    contact.object_point_turned[0] = st(t13);
    let t14 = int(contact.contact_point[1] as i32);
    let t15 = numbers.k65536.ld();
    let t16 = t14 / t15;
    contact.object_point_turned[1] = st(t16);
    let t17 = int(contact.contact_point[2] as i32);
    let t18 = numbers.k65536.ld();
    let t19 = t17 / t18;
    contact.object_point_turned[2] = st(t19);
}

#[cfg(test)]
mod tests;
