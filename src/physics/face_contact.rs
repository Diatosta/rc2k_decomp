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
//! Caller: `physics::record_face_contacts`. Refuses a kind past the 0x4B0 material records
//! (but for an unloaded slot's, `contacts::UNSET_KIND`: stage 22 has such a face), and what its
//! calls refuse.

use crate::effects::cell_x87::CellX87;
use crate::errors::Refused;
use crate::math::x87::compare_80;
use crate::math::x87::int_hypot;
use crate::math::x87_float::int;
use crate::math::x87_float::to_int;
use crate::physics::car_store::Car;
use crate::physics::contact_force::contact_force;
use crate::physics::contact_store::ContactWork;
use crate::physics::flow::{ZERO_BITS as ZERO, copy_cells, fill_cells, negate};
use crate::physics::step_families::ContactInputs;
use crate::physics::touch_face::touch_face;
use crate::physics::vector_length::vector_length;
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
    c.contact.face_number.set(number);
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
            c.contact.contact_kind.set(0xFF);
            return touch(c, scene);
        }
        FaceObject::Object(object) => object,
    };
    c.contact.face_object.set(object);
    let kind = scene.objects.tables.get().object_kind(object);
    if let Err(e) = contacts::check_material(kind) {
        crate::framework::log(&format!(
            "physics::face_contact: face {}: object {object}, kind {kind:#x}, car {} at {:?}",
            face,
            c.work.current_car.get(),
            c.car
                .position
                .iter()
                .map(|c| c.get() as i32)
                .collect::<Vec<_>>(),
        ));
        return Err(e.into());
    }
    // An unset kind wraps to the record before the first.
    c.contact.contact_kind.set(
        crate::scene::polygon::material_run_byte(
            &c.shared.buffers.reads.get().polygon_tables,
            kind,
            0x1C,
        )?
        .into(),
    );
    if contacts::object_tried(face_byte(0x25)?, c.work.stage_direction.get(), object).is_err() {
        return Ok(());
    }
    for (k, o) in [8u32, 0xC, 0x10].iter().enumerate() {
        scene.object_work().angles[k].set(contacts::turn_word(face_byte(*o)?));
    }
    scene.object_work().size.set(face_byte(0x18)?);
    // The face is both the record and the object here.
    let (inside, point) = { crate::physics::scene_point_hits_car::scene_point_hits_car(c, face) }?;
    if !inside {
        if !contacts::tries_points(c.contact.contact_kind.get()) {
            return Ok(());
        }
        let dx = crate::race::scene_points::point_word(
            c.course.reads(),
            &crate::race::scene_points::no_own(),
            point,
            0,
        )?
        .wrapping_add(face_dword(0x30)?)
        .wrapping_sub(c.car.position[0].get());
        let dz = crate::race::scene_points::point_word(
            c.course.reads(),
            &crate::race::scene_points::no_own(),
            point,
            2,
        )?
        .wrapping_add(face_dword(0x38)?)
        .wrapping_sub(c.car.position[2].get());
        let distance = int_hypot(dx as i32, dz as i32) as u32;
        scene.drawing.scene.get().scratch.set(distance);
        let (reach, _) = contacts::reach(
            scene.objects.slots.get().radii[c.contact.face_object.get() as usize].get(),
            scene.object_work().size.get(),
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
    x87_1(c.contact, c.numbers);
    c.contact
        .face_angle
        .set(c.contact.face_angle.get().wrapping_add(0x80) & 0xFF);
    x87_2(c.contact, c.car, c.numbers);
    match contacts::inside(c.contact.contact_kind.get()) {
        Inside::PushBack => {}
        Inside::Touch => return touch(c, scene),
        Inside::Nothing => return Ok(()),
    }
    { crate::physics::restore_saved_car::restore_saved_car(c.car, c.tables, c.contact) }?;
    { crate::physics::feel_ground::feel_ground(&c.ground, scene) }?;
    touch(c, scene)?;
    // Pushed back against the motion, level.
    copy_cells(&c.scratch.work_vector, &c.car.velocity);
    negate(&c.scratch.work_vector[0]);
    negate(&c.scratch.work_vector[1]);
    negate(&c.scratch.work_vector[2]);
    if compare_80(vector_length(&c.scratch.work_vector), ZERO) & 0x4000 != 0 {
        fill_cells(&c.scratch.work_vector, 0.0);
    }
    c.scratch.work_vector[1].set(0);
    normalise_work_vector(c.numbers, c.scratch, scene);
    copy_cells(&c.contact.contact_normal, &c.scratch.work_vector);
    copy_cells(&c.contact.contact_car_speed, &c.car.velocity);
    copy_cells(&c.contact.contact_turn_speed, &c.car.spin);
    copy_cells(&c.contact.contact_point, &c.contact.object_point_scaled);
    c.contact.contact_point[1].set(0.0);
    c.contact.contact_slide_limit.set(0.0);
    contact_force(
        c.contact,
        c.numbers,
        c.tables,
        c.car,
        &c.tables.contacts[0],
        scene,
    );
    Ok(())
}

// ---- the x87 arithmetic ----

/// One straight x87 run as plain Rust over `X`.
fn x87_1(contact: &ContactWork, numbers: &Numbers) {
    let t0 = int(contact.object_point_offset[0].get() as i32);
    let t1 = int(contact.object_point_offset[2].get() as i32);
    let t2 = t0.atan2(t1);
    let t3 = numbers.k256.ld();
    let t4 = t2 * t3;
    let t5 = numbers.one_over_two_pi.ld();
    let t6 = t4 * t5;
    contact.face_angle.set(to_int(t6) as u32);
}

/// One straight x87 run as plain Rust over `X`.
fn x87_2(contact: &ContactWork, car: &Car, numbers: &Numbers) {
    let t0 = car.velocity[0].ld();
    let t1 = t0 * t0;
    let t2 = car.velocity[2].ld();
    let t3 = t2 * t2;
    let t4 = car.velocity[1].ld();
    let t5 = t4 * t4;
    let t6 = t3 + t5;
    let t7 = t1 + t6;
    let t8 = t7.sqrt();
    let t9 = numbers.k65536.ld();
    let t10 = t8 * t9;
    contact.face_speed.set(to_int(t10) as u32);
    let t11 = int(crate::physics::flow::whole(&contact.contact_point[0]));
    let t12 = numbers.k65536.ld();
    let t13 = t11 / t12;
    contact.object_point_turned[0].st(t13);
    let t14 = int(crate::physics::flow::whole(&contact.contact_point[1]));
    let t15 = numbers.k65536.ld();
    let t16 = t14 / t15;
    contact.object_point_turned[1].st(t16);
    let t17 = int(crate::physics::flow::whole(&contact.contact_point[2]));
    let t18 = numbers.k65536.ld();
    let t19 = t17 / t18;
    contact.object_point_turned[2].st(t19);
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    fn blocks() -> Vec<Block> {
        vec![
            Block {
                name: "face_contact line 111",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x1C5DD4, In::F),
                        (0x1C5E2C, In::F),
                        (0x2D3B4C, In::I),
                        (0x2D3B54, In::I),
                    ],
                    writes: &[(0x2D3B70, Out::I)],
                }],
                ulps: 0,
                plain: |_| {
                    x87_1(
                        crate::test_support::own::race().physics.contact.get(),
                        crate::test_support::own::race().course.numbers.get(),
                    );
                },
            },
            Block {
                name: "face_contact line 121",
                operands: vec![Operand::RaceBlock {
                    reads: &[
                        (0x1C5D98, In::F),
                        (0x2D3090, In::I),
                        (0x2D3094, In::I),
                        (0x2D3098, In::I),
                        (0x31A170, In::F),
                        (0x31A174, In::F),
                        (0x31A178, In::F),
                    ],
                    writes: &[
                        (0x2D3B58, Out::F),
                        (0x2D3B5C, Out::F),
                        (0x2D3B60, Out::F),
                        (0x2D3B74, Out::I),
                    ],
                }],
                ulps: 0,
                plain: |_| {
                    x87_2(
                        crate::test_support::own::race().physics.contact.get(),
                        crate::test_support::own::race().shared.car(),
                        crate::test_support::own::race().course.numbers.get(),
                    );
                },
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse("physics.face_contact", &blocks(), 1200, 100);
    }
}
