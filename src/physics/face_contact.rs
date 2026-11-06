//! `physics::face_contact` (0x45E3BF): one course face (ebx, `g_courseFaces` + 0x40 n)
//! tried against the work car. `dword_6D3218` = n. Faces hidden by kind
//! (`scene::kind_hidden_at`) and faces whose first byte is not 0xFF (not an object's)
//! are left. An object byte (+1) of 0x3B: the kind 0xFF, `physics::touch_face`. Else the
//! object (`dword_6D321C`), its contact kind `dword_6D3624` from its material record
//! (`g_objectMaterials` + 0x80 `g_objectKinds[n]`, byte +0x1C); left when byte +0x25
//! bit 0 is set, when its bits 1 .. 2 (a direction + 1) are not 0 and not
//! `g_stageDirection` + 1, or when the object is 0x38 or more but for 0x3A. Its turn
//! (+8, +0xC, +0x10 bytes << 8 into `dword_63503C`, `635040`, `635044`) and scale (+0x18
//! into `dword_635038`); `physics::scene_point_hits_car` (ebp the face). Not inside: for
//! kinds 9, 8, 6, 5, when the point's distance from the car (x, z) is within the object's
//! reach (`dword_634BA0[n]` x the scale >> 4, + 0x28000, by the sign of the `cmp`),
//! `physics::object_hits_car`. Inside either way: the contact's angle (`fpatan`, into
//! `dword_6D3B70`, + 0x80 & 0xFF), the car's speed x 65536 (`dword_6D3B74`) and the
//! point / 65536 (`flt_6D3B58` ..); then by kind: 2, 7, 3, 8, 4 `touch_face`; 6, 9, 5, 1,
//! 10 `physics::restore_saved_car`, `physics::feel_ground`, `touch_face` and the push
//! back: minus the car's speed, level (y 0), made one long (`race::normalise_work_vector`)
//! as the normal, the speeds and the point (y 0) taken, no slide limit,
//! `physics::contact_force` into contact record 0.
//!
//! In:  ebx the face
//! Out: the registers as the steps and calls leave them (eax's low byte from the face's
//!      bytes where the original loads `al`); no flags read (the caller goes on)
//!
//! Caller: `physics::record_face_contacts`. Refuses a kind past the 0x4B0 material records
//! (but for an unloaded slot's, `contacts::UNSET_KIND`: stage 22 has such a face),
//! and what its calls refuse.
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::errors::Refused;
use crate::math::x87::int_hypot;
use crate::math::x87_float::int;
use crate::math::x87_float::ld;
use crate::math::x87_float::st;
use crate::math::x87_float::to_int;
use crate::math::x87_run::compare_80;
use crate::physics::contact_force::contact_force;
use crate::physics::flow::{copy_cells, fill_cells, negate};
use crate::physics::touch_face::touch_face;
use crate::physics::vector_length::vector_length;
use crate::race::norms::normalise_work_vector;
use crate::race::owner::Race;
use crate::scene::owner::scene;
use rc2k_core::physics::face_contacts::{self as contacts, FaceObject, Inside};

fn touch(race: &Race) -> Result<(), Refused> {
    { touch_face(race) }?;
    Ok(())
}

/// The steps for face `number` (ebx). The registers the original leaves are read by none
/// of its callers.
// Converted: `rc2k_core::physics::face_contacts` (the x87 steps stay).
pub fn steps(race: &Race, number: u32) -> Result<(), Refused> {
    let scene = scene();
    race.physics.contact.face_number.set(number);
    let face = number;
    let face_byte = |offset| -> Result<u32, Refused> {
        Ok(u32::from(crate::race::course_places::face_field::<u8>(
            face,
            offset,
            "a face's byte",
        )?))
    };
    let face_dword = |offset| -> Result<u32, Refused> {
        Ok(crate::race::course_places::face_field::<u32>(
            face,
            offset,
            "a face's dword",
        )?)
    };
    if crate::scene::kind_hidden_at::kind_hidden_at(face)? {
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
            race.physics.contact.contact_kind.set(0xFF);
            return touch(race);
        }
        FaceObject::Object(object) => object,
    };
    race.physics.contact.face_object.set(object);
    let kind = scene.objects.tables.object_kind(object);
    if let Err(e) = contacts::check_material(kind) {
        crate::framework::log(&format!(
            "physics::face_contact: face {}: object {object}, kind {kind:#x}, car {} at {:?}",
            face,
            race.shared.work.current_car.get(),
            race.shared
                .car
                .position
                .iter()
                .map(|c| c.get() as i32)
                .collect::<Vec<_>>(),
        ));
        return Err(e.into());
    }
    // An unset kind wraps to the record before the first, as the original reads it.
    race.physics
        .contact
        .contact_kind
        .set(crate::scene::polygon::material_byte(kind, 0x1C)?.into());
    if contacts::object_tried(
        face_byte(0x25)?,
        race.shared.work.stage_direction.get(),
        object,
    )
    .is_err()
    {
        return Ok(());
    }
    for (k, o) in [8u32, 0xC, 0x10].iter().enumerate() {
        scene.drawing.object_work.angles[k].set(contacts::turn_word(face_byte(*o)?));
    }
    scene.drawing.object_work.size.set(face_byte(0x18)?);
    // esi and ebp both the face here.
    let (inside, point) =
        { crate::physics::scene_point_hits_car::scene_point_hits_car(race, face) }?;
    if !inside {
        if !contacts::tries_points(race.physics.contact.contact_kind.get()) {
            return Ok(());
        }
        let dx = crate::race::scene_points::point_word(point, 0)?
            .wrapping_add(face_dword(0x30)?)
            .wrapping_sub(race.shared.car.position[0].get());
        let dz = crate::race::scene_points::point_word(point, 2)?
            .wrapping_add(face_dword(0x38)?)
            .wrapping_sub(race.shared.car.position[2].get());
        let distance = int_hypot(dx as i32, dz as i32) as u32;
        scene.drawing.scene.scratch.set(distance);
        let (reach, _) = contacts::reach(
            scene.objects.slots.radii[race.physics.contact.face_object.get() as usize].get(),
            scene.drawing.object_work.size.get(),
        );
        if contacts::out_of_reach(reach, distance) {
            return Ok(());
        }
        // esi the scene point, ebp the face.
        if !{ crate::physics::object_hits_car::object_hits_car(race, point, face) }? {
            return Ok(());
        }
    }
    // 0x45E523: inside.
    x87_1(race);
    race.physics
        .contact
        .face_angle
        .set(race.physics.contact.face_angle.get().wrapping_add(0x80) & 0xFF);
    x87_2(race);
    match contacts::inside(race.physics.contact.contact_kind.get()) {
        Inside::PushBack => {}
        Inside::Touch => return touch(race),
        Inside::Nothing => return Ok(()),
    }
    { crate::physics::restore_saved_car::restore_saved_car(race) }?;
    { crate::physics::feel_ground::feel_ground(race) }?;
    touch(race)?;
    // 0x45E62A: pushed back against the motion, level.
    copy_cells(&race.frame.scratch.work_vector, &race.shared.car.velocity);
    negate(&race.frame.scratch.work_vector[0]);
    negate(&race.frame.scratch.work_vector[1]);
    negate(&race.frame.scratch.work_vector[2]);
    if compare_80(
        vector_length(&race.frame.scratch.work_vector),
        race.course.numbers.zero.bits(),
    ) & 0x4000
        != 0
    {
        fill_cells(&race.frame.scratch.work_vector, 0);
    }
    race.frame.scratch.work_vector[1].set(0);
    normalise_work_vector();
    copy_cells(
        &race.physics.contact.contact_normal,
        &race.frame.scratch.work_vector,
    );
    copy_cells(
        &race.physics.contact.contact_car_speed,
        &race.shared.car.velocity,
    );
    copy_cells(
        &race.physics.contact.contact_turn_speed,
        &race.shared.car.spin,
    );
    copy_cells(
        &race.physics.contact.contact_point,
        &race.physics.contact.object_point_scaled,
    );
    race.physics.contact.contact_point[1].set_bits(0);
    race.physics.contact.contact_slide_limit.set_bits(0);
    contact_force(race, &race.shared.physics_tables.contacts[0]);
    Ok(())
}

// ---- the arithmetic that was x87 assembly ----

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// `int [rc + 0x2D3B70] = (atan2(int [rc + 0x2D3B4C], int [rc + 0x2D3B54]) * [rc + 0x1C5E2C]) * [rc + 0x1C5DD4]`
fn x87_1(race: &Race) {
    let t0 = int(race.physics.contact.object_point_offset[0].bits() as i32);
    let t1 = int(race.physics.contact.object_point_offset[2].bits() as i32);
    let t2 = t0.atan2(t1);
    let t3 = ld(race.course.numbers.k256.bits());
    let t4 = t2 * t3;
    let t5 = ld(race.course.numbers.one_over_two_pi.bits());
    let t6 = t4 * t5;
    race.physics.contact.face_angle.set_bits(to_int(t6) as u32);
}

/// An x87 run of the original, in plain Rust over `X`, over the memory at the operands'
/// addresses (`[base + offset]`: a float; `int [..]`: a whole number; `ext [..]`: 80 bits):
/// (a long formula: see the instructions in the tests' `asm_reference`)
/// `[rc + 0x2D3B58] = int [rc + 0x2D3090] / [rc + 0x1C5D98]`
/// `[rc + 0x2D3B5C] = int [rc + 0x2D3094] / [rc + 0x1C5D98]`
/// `[rc + 0x2D3B60] = int [rc + 0x2D3098] / [rc + 0x1C5D98]`
fn x87_2(race: &Race) {
    let t0 = ld(race.shared.car.velocity[0].bits());
    let t1 = t0 * t0;
    let t2 = ld(race.shared.car.velocity[2].bits());
    let t3 = t2 * t2;
    let t4 = ld(race.shared.car.velocity[1].bits());
    let t5 = t4 * t4;
    let t6 = t3 + t5;
    let t7 = t1 + t6;
    let t8 = t7.sqrt();
    let t9 = ld(race.course.numbers.k65536.bits());
    let t10 = t8 * t9;
    race.physics.contact.face_speed.set_bits(to_int(t10) as u32);
    let t11 = int(race.physics.contact.contact_point[0].bits() as i32);
    let t12 = ld(race.course.numbers.k65536.bits());
    let t13 = t11 / t12;
    race.physics.contact.object_point_turned[0].set_bits(st(t13));
    let t14 = int(race.physics.contact.contact_point[1].bits() as i32);
    let t15 = ld(race.course.numbers.k65536.bits());
    let t16 = t14 / t15;
    race.physics.contact.object_point_turned[1].set_bits(st(t16));
    let t17 = int(race.physics.contact.contact_point[2].bits() as i32);
    let t18 = ld(race.course.numbers.k65536.bits());
    let t19 = t17 / t18;
    race.physics.contact.object_point_turned[2].set_bits(st(t19));
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions, one function per run.
    reference!(ref_0(rc) "fild dword ptr [{rc} + 0x2D3B4C]", "fild dword ptr [{rc} + 0x2D3B54]", ".byte 0xD9, 0xF3", "fmul dword ptr [{rc} + 0x1C5E2C]", "fmul dword ptr [{rc} + 0x1C5DD4]", "fistp dword ptr [{rc} + 0x2D3B70]");
    reference!(ref_1(rc) "fld dword ptr [{rc} + 0x31A170]", ".byte 0xD8, 0xC8", "fld dword ptr [{rc} + 0x31A178]", ".byte 0xD8, 0xC8", "fld dword ptr [{rc} + 0x31A174]", ".byte 0xD8, 0xC8", ".byte 0xDE, 0xC1", ".byte 0xDE, 0xC1", ".byte 0xD9, 0xFA", "fmul dword ptr [{rc} + 0x1C5D98]", "fistp dword ptr [{rc} + 0x2D3B74]", "fild dword ptr [{rc} + 0x2D3090]", "fdiv dword ptr [{rc} + 0x1C5D98]", "fstp dword ptr [{rc} + 0x2D3B58]", "fild dword ptr [{rc} + 0x2D3094]", "fdiv dword ptr [{rc} + 0x1C5D98]", "fstp dword ptr [{rc} + 0x2D3B5C]", "fild dword ptr [{rc} + 0x2D3098]", "fdiv dword ptr [{rc} + 0x1C5D98]", "fstp dword ptr [{rc} + 0x2D3B60]");

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
                reference: |a| unsafe { ref_0(a[0]) },
                plain: |_| x87_1(crate::race::owner::race()),
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
                reference: |a| unsafe { ref_1(a[0]) },
                plain: |_| x87_2(crate::race::owner::race()),
            },
        ]
    }

    #[test]
    fn x87_blocks_matches_the_assembly() {
        sparse(&blocks(), 1200, 100);
    }
}
