//! One step of the stage's moving faces, the lit faces (the course faces the car has touched:
//! `physics::touch_face` lists them): roadside objects that react for a while by their object's
//! kind, blinking (1, 9), rising (2, 3, 8) or being knocked away along the ground (4); object
//! 0x3B counts down. Each face's pace comes from its distance to car block 0 (closer is
//! faster). A face leaves the list by the last one taking its place, and the next one is looked
//! at (so the one moved in waits a step). Refuses (before anything) more than 0x100 lit faces and
//! a face number past the course's face count; `find_ground`'s refusals mid-way (what it wrote
//! is put back first).

use crate::errors::{BadInput, Refused};
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::owner_physics::Physics;
use rc2k_core::drive::lit_faces::{
    COUNTDOWN_OBJECT, FaceBytes, RISE_TOP, align, blink, count_down, fall_turn, pace, rise,
};

const MOST: u32 = 0x100;

/// Field `o` of course face `f` (by number), and its writes.
fn at(course: &Course, f: u32, o: u32) -> Result<u32, BadInput> {
    crate::race::course_places::face_field(course.reads(), f, o)
}

fn put(course: &Course, f: u32, o: u32, v: u32) -> Result<(), BadInput> {
    crate::race::course_places::write_face_field(course.reads(), f, o, v)
}

fn byte(course: &Course, f: u32, o: u32) -> Result<u8, BadInput> {
    crate::race::course_places::face_field(course.reads(), f, o)
}

fn set_byte(course: &Course, f: u32, o: u32, v: u8) -> Result<(), BadInput> {
    crate::race::course_places::write_face_field(course.reads(), f, o, v)
}

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

/// The lit face list's entry `k` (0x100 of them; the count is checked below).
fn list(physics: &Physics, k: u32) -> &crate::memory::session::PlainDword {
    &physics.contact.get().lit_faces[k as usize]
}

fn refusal(race: &Race) -> Option<&'static str> {
    let count: u32 = race.physics.contact.get().lit_face_count.get();
    if count > MOST {
        return Some("more than 0x100 lit faces (past g_litFaces)");
    }
    let faces: u32 = race.course.course.get().course_face_count.get();
    for k in 0..count {
        if list(&race.physics, k).get() >= faces {
            return Some("a lit face past g_courseFaceCount");
        }
    }
    None
}

fn animate_at(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    if let Some(why) = refusal(race) {
        return Err(BadInput(why).into());
    }
    if race.physics.contact.get().lit_face_count.get() == 0 {
        return Ok(());
    }

    animate_faces(race, scene)
}

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
            let dx = crate::race::scene_points::point_word(
                race.course.reads(),
                &crate::race::scene_points::no_own(),
                point_number,
                0,
            )?
            .wrapping_sub(race.car_blocks().cars[0].position[0].get());
            scene.drawing.scene.get().scratch.set(dx);
            let dz = crate::race::scene_points::point_word(
                race.course.reads(),
                &crate::race::scene_points::no_own(),
                point_number,
                2,
            )?
            .wrapping_sub(race.car_blocks().cars[0].position[2].get());
            scene.drawing.scene.get().scratch.set(dz);
            let d = crate::math::x87::int_hypot(dx as i32, dz as i32) as u32;
            let step = pace(d);
            let kind_of: u32 = scene.objects.tables.get().object_kind(object as u32);
            // The material's kind byte: the low byte of its eighth word.
            let kind =
                crate::scene::polygon::material_run_byte(race.polygon_tables(), kind_of, 0x1C)?;
            match kind {
                1 | 9 => {
                    blink(&mut bytes, |n| {
                        // `blink_levels`: the step is below 0x18 (`BLINK_STEPS`).
                        (race.physics.contact.get().blink_levels[n as usize / 4].get()
                            >> (8 * (n % 4))) as u8
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
                        race.engine_tables(),
                        0,
                        point_number as i32,
                        distance as i32,
                        angle,
                    );
                    race.physics
                        .contact
                        .get()
                        .lit_face_saved_30
                        .set(at(&race.course, f, 0x30)?);
                    race.physics
                        .contact
                        .get()
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
                    let ground_x = crate::race::scene_points::point_word(
                        race.course.reads(),
                        &crate::race::scene_points::no_own(),
                        p,
                        0,
                    )?
                    .wrapping_add(at(&race.course, f, 0x30)?);
                    let ground_z = crate::race::scene_points::point_word(
                        race.course.reads(),
                        &crate::race::scene_points::no_own(),
                        p,
                        2,
                    )?
                    .wrapping_add(at(&race.course, f, 0x38)?);
                    {
                        let mut search = race.physics.ground_search().borrow_mut();
                        search.x = ground_x;
                        search.z = ground_z;
                        search.last = race.car_blocks().cars[0].probe_point_face[4].get();
                    }
                    crate::physics::find_ground::find_ground(
                        race.physics.ground_probe.get(),
                        race.physics.ground_search(),
                        &race.course,
                    )?;
                    put(
                        &race.course,
                        f,
                        0x34,
                        race.physics.ground_search().borrow().height.wrapping_sub(
                            crate::race::scene_points::point_word(
                                race.course.reads(),
                                &crate::race::scene_points::no_own(),
                                p,
                                1,
                            )?,
                        ),
                    )?;
                    if race.physics.ground_search().borrow().slot == u32::MAX {
                        put(
                            &race.course,
                            f,
                            0x30,
                            race.physics.contact.get().lit_face_saved_30.get(),
                        )?;
                        put(
                            &race.course,
                            f,
                            0x38,
                            race.physics.contact.get().lit_face_saved_38.get(),
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
            let last = race
                .physics
                .contact
                .get()
                .lit_face_count
                .get()
                .wrapping_sub(1);
            list(&race.physics, k).set(list(&race.physics, last).get());
            race.physics.contact.get().lit_face_count.set(last);
        }
        k += 1;
        if k >= race.physics.contact.get().lit_face_count.get() {
            return Ok(());
        }
    }
}

pub fn animate_lit_faces(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    animate_at(race, scene)
}
