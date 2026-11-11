//! `drive::animate_lit_faces` (0x45EB06): one step of the stage's moving faces, those in
//! `g_litFaces` (0x6D3220, `g_litFaceCount` of them). For each (face record at
//! `g_courseFaces` + 0x40 n):
//! - object 0x3B (byte +1): a countdown at +0x11; at 0 the face leaves the list;
//! - else its distance from car block 0 (+0x134 / +0x13C: `CarsAndReplay.car_blocks.cars[0].position[0]`, `CarsAndReplay.car_blocks.cars[0].position[2]`;
//!   `fistp` of the root of the squares, through `scratch`) into `lit_face_distance`, and
//!   a pace 1 .. 17 from it (closer is faster) into `lit_face_pace`; then by its object's
//!   kind (`Buffers.polygon_tables.object_materials[0].words[7][g_objectKinds[object] * 0x80]`):
//!   - 1, 9: +0x11 counts up to 0x18 (then cleared, and the face leaves), +0x13 from
//!     `blink_levels`;
//!   - 2: +0x13 up by the pace, leaving at 0x30;
//!   - 3, 8: +9 stepped (the pace times 4) towards making +8 + +9 + +0x12 a multiple of
//!     0x80; once it is, as 2 up to 0x40;
//!   - 4: while +0x11 is not 0, the face's offset (+0x30, +0x38) moved by
//!     `math::rotate_point_about_y` of (0, +0x11 << 8) by (+0x12 + 0x80) << 24, its height
//!     (+0x34) set on the ground under it (`physics::find_ground`, last face
//!     `CarsAndReplay.car_blocks.cars[0].probe_point_face[4]`; no ground puts the offset back), +0x11 less 1; at 0 it leaves;
//!   - others: nothing.
//!
//! A face leaves by the list's last taking its place (the count less 1), and the next
//! one is looked at (so the one moved in waits a step). These are the course faces the
//! car has touched (`physics::touch_face` lists them): roadside objects that react for a
//! while by their object's kind, blinking (1, 9), rising (2, 3, 8) or being knocked away
//! along the ground (4).
//!
//! Caller: `drive::animate_stage` (0x45E9CA). Refuses (before anything) more than 0x100 lit faces
//! (the list's room) and a face number past `g_courseFaceCount`; `find_ground`'s refusals mid-way
//! (what it wrote is put back first).

use crate::errors::{BadInput, Refused};
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::owner_physics::Physics;
use rc2k_core::drive::lit_faces::{
    COUNTDOWN_OBJECT, FaceBytes, RISE_TOP, align, blink, count_down, fall_turn, pace, rise,
};

const MOST: u32 = 0x100;

/// Field `o` of course face `f` (by number), and its writes.
fn at(course: &Course, f: u32, o: u32) -> Result<u32, BadInput> {
    crate::race::course_places::face_field(course, f, o)
}

fn put(course: &Course, f: u32, o: u32, v: u32) -> Result<(), BadInput> {
    crate::race::course_places::write_face_field(course, f, o, v)
}

fn byte(course: &Course, f: u32, o: u32) -> Result<u8, BadInput> {
    crate::race::course_places::face_field(course, f, o)
}

fn set_byte(course: &Course, f: u32, o: u32, v: u8) -> Result<(), BadInput> {
    crate::race::course_places::write_face_field(course, f, o, v)
}

/// The face's animated bytes.
fn load(course: &Course, f: u32) -> Result<FaceBytes, BadInput> {
    Ok(FaceBytes {
        b8: byte(course, f, 8)?,
        b9: byte(course, f, 9)?,
        b11: byte(course, f, 0x11)?,
        b12: byte(course, f, 0x12)?,
        b13: byte(course, f, 0x13)?,
    })
}

fn store(course: &Course, f: u32, b: &FaceBytes) -> Result<(), BadInput> {
    for (o, v) in [(9, b.b9), (0x11, b.b11), (0x13, b.b13)] {
        set_byte(course, f, o, v)?;
    }
    Ok(())
}

/// `g_litFaces`[k] (0x100 of them; the count is checked below).
fn list(physics: &Physics, k: u32) -> &crate::session::Dword {
    &physics.contact.lit_faces[k as usize]
}

/// The refusals, worked out before anything is written.
fn refusal(race: &Race) -> Option<&'static str> {
    let count: u32 = race.physics.contact.lit_face_count.get();
    if count > MOST {
        return Some("more than 0x100 lit faces (past g_litFaces)");
    }
    let faces: u32 = race.course.course.course_face_count.get();
    for k in 0..count {
        if list(&race.physics, k).get() >= faces {
            return Some("a lit face past g_courseFaceCount");
        }
    }
    None
}

/// The Rust core.
pub fn animate_at(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    if let Some(why) = refusal(race) {
        return Err(BadInput(why).into());
    }
    if race.physics.contact.lit_face_count.get() == 0 {
        return Ok(());
    }

    animate_faces(race, scene)
}

// Converted: `rc2k_core::drive::lit_faces` (all but kind 4's turn and ground).
fn animate_faces(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let mut k = 0;
    loop {
        let f = list(&race.physics, k).get();
        let mut bytes = load(&race.course, f)?;
        let leave = 'face: {
            let object = byte(&race.course, f, 1)?;
            if object == COUNTDOWN_OBJECT {
                break 'face count_down(&mut bytes);
            }
            let point_number = at(&race.course, f, 4)?;
            let dx = crate::race::scene_points::point_word(&race.course, point_number, 0)?
                .wrapping_sub(CAR_BLOCKS.cars[0].position[0].get());
            scene.drawing.scene.scratch.set(dx);
            let dz = crate::race::scene_points::point_word(&race.course, point_number, 2)?
                .wrapping_sub(CAR_BLOCKS.cars[0].position[2].get());
            scene.drawing.scene.scratch.set(dz);
            let d = crate::math::x87::int_hypot(dx as i32, dz as i32) as u32;
            race.physics.contact.lit_face_distance.set(d);
            let step = pace(d);
            race.physics.contact.lit_face_pace.set(step);
            let kind_of: u32 = scene.objects.tables.object_kind(object as u32);
            // The material's byte +0x1C (the low byte of its eighth word).
            let kind = crate::scene::polygon_store::POLYGON_TABLES
                .object_materials
                .get(kind_of as usize)
                .ok_or(crate::errors::BadInput("an object kind past the materials"))?
                .words[7]
                .get() as u8;
            match kind {
                1 | 9 => {
                    blink(&mut bytes, |n| {
                        // `blink_levels`: the step is below 0x18 (`BLINK_STEPS`).
                        (race.physics.contact.blink_levels[n as usize / 4].get() >> (8 * (n % 4)))
                            as u8
                    })
                    .is_none()
                }
                2 => rise(&mut bytes, step, RISE_TOP).0,
                3 | 8 => align(&mut bytes, step).0,
                4 => {
                    let Some((distance, angle)) = fall_turn(&bytes) else {
                        break 'face true;
                    };
                    let [x, _, z, _] = crate::math::rotate_point_about_y::rotate_point_about_y(
                        0,
                        point_number as i32,
                        distance as i32,
                        angle,
                    );
                    race.physics
                        .contact
                        .lit_face_saved_30
                        .set(at(&race.course, f, 0x30)?);
                    race.physics
                        .contact
                        .lit_face_saved_38
                        .set(at(&race.course, f, 0x38)?);
                    put(
                        &race.course,
                        f,
                        0x30,
                        at(&race.course, f, 0x30)?.wrapping_add(x),
                    )?;
                    put(
                        &race.course,
                        f,
                        0x38,
                        at(&race.course, f, 0x38)?.wrapping_add(z),
                    )?;
                    let p = at(&race.course, f, 4)?;
                    race.physics.ground.ground_x.set(
                        crate::race::scene_points::point_word(&race.course, p, 0)?
                            .wrapping_add(at(&race.course, f, 0x30)?),
                    );
                    race.physics.ground.ground_z.set(
                        crate::race::scene_points::point_word(&race.course, p, 2)?
                            .wrapping_add(at(&race.course, f, 0x38)?),
                    );
                    race.course
                        .course
                        .ground_face_last
                        .set(CAR_BLOCKS.cars[0].probe_point_face[4].get());
                    crate::physics::find_ground::find_ground(
                        &race.physics.ground_probe,
                        &race.physics.ground,
                        &race.course,
                        scene,
                    )?;
                    put(
                        &race.course,
                        f,
                        0x34,
                        race.physics.ground.ground_height.get().wrapping_sub(
                            crate::race::scene_points::point_word(&race.course, p, 1)?,
                        ),
                    )?;
                    if race.course.course.ground_slot.get() == u32::MAX {
                        put(
                            &race.course,
                            f,
                            0x30,
                            race.physics.contact.lit_face_saved_30.get(),
                        )?;
                        put(
                            &race.course,
                            f,
                            0x38,
                            race.physics.contact.lit_face_saved_38.get(),
                        )?;
                    }
                    bytes.b11 = bytes.b11.wrapping_sub(1);
                    false
                }
                _ => false,
            }
        };
        store(&race.course, f, &bytes)?;
        if leave {
            let last = race.physics.contact.lit_face_count.get().wrapping_sub(1);
            list(&race.physics, k).set(list(&race.physics, last).get());
            race.physics.contact.lit_face_count.set(last);
        }
        k += 1;
        if k >= race.physics.contact.lit_face_count.get() {
            return Ok(());
        }
    }
}

pub fn animate_lit_faces(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    animate_at(race, scene)
}
