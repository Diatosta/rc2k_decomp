//! One step of the stage's moving faces, the lit faces (the course faces the car has touched:
//! `physics::touch_face` lists them): roadside objects that react for a while by their object's
//! kind, blinking (1, 9), rising (2, 3, 8) or being knocked away along the ground (4); object
//! 0x3B counts down. Each face's pace comes from its distance to car block 0 (closer is
//! faster). A face leaves the list by the last one taking its place, and the next one is looked
//! at (so the one moved in waits a step). Refuses (before anything) more than 0x100 lit faces and
//! a face number past the course's face count; `find_ground`'s refusals mid-way (what it wrote
//! is put back first).

use crate::errors::{BadInput, Refused};
use crate::memory::plain_floats::FloatRow;
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use crate::race::owner_physics::Physics;
use rc2k_core::drive::lit_faces::{
    COUNTDOWN_OBJECT, FaceBytes, RISE_TOP, align, blink, count_down, fall_turn, pace, rise,
};

const MOST: u32 = 0x100;

/// The blink's brightness levels by step, four to a word (the low byte first).
const BLINK_LEVELS: [u32; 6] = [
    0xFEFD_FEFF,
    0x0201_00FF,
    0x0001_0203,
    0x00FF_FEFF,
    0x0001_0201,
    0x0001_00FF,
];

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
fn listed(physics: &Physics, k: u32) -> u32 {
    physics.lit_faces.borrow().faces[k as usize]
}

fn refusal(race: &Race) -> Option<&'static str> {
    let count: u32 = race.physics.lit_faces.borrow().count;
    if count > MOST {
        return Some("more than 0x100 lit faces (past g_litFaces)");
    }
    let faces: u32 = race.course.course.borrow().course_face_count;
    for k in 0..count {
        if listed(&race.physics, k) >= faces {
            return Some("a lit face past g_courseFaceCount");
        }
    }
    None
}

fn animate_at(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    if let Some(why) = refusal(race) {
        return Err(BadInput(why).into());
    }
    if race.physics.lit_faces.borrow().count == 0 {
        return Ok(());
    }

    animate_faces(race, scene)
}

fn animate_faces(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let mut k = 0;
    loop {
        let f = listed(&race.physics, k);
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
            .wrapping_sub(race.car_blocks().cars[0].position.bits_at(0));
            scene.draw_work_mut().scratch = dx;
            let dz = crate::race::scene_points::point_word(
                race.course.reads(),
                &crate::race::scene_points::no_own(),
                point_number,
                2,
            )?
            .wrapping_sub(race.car_blocks().cars[0].position.bits_at(2));
            scene.draw_work_mut().scratch = dz;
            let d = crate::math::x87::int_hypot(dx as i32, dz as i32) as u32;
            let step = pace(d);
            let kind_of: u32 = scene.objects.tables.borrow().object_kind(object as u32);
            // The material's kind byte: the low byte of its eighth word.
            let kind =
                crate::scene::polygon::material_run_byte(race.polygon_tables(), kind_of, 0x1C)?;
            match kind {
                1 | 9 => {
                    blink(&mut bytes, |n| {
                        // The step is below 0x18 (`BLINK_STEPS`).
                        (BLINK_LEVELS[n as usize / 4] >> (8 * (n % 4))) as u8
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
                    let saved_x = at(&race.course, f, 0x30)?;
                    let saved_z = at(&race.course, f, 0x38)?;
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
                        search.last = race.car_blocks().cars[0].probe_point_face[4];
                    }
                    crate::physics::find_ground::find_ground(
                        race.physics.ground_probe(),
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
                        put(&race.course, f, 0x30, saved_x)?;
                        put(&race.course, f, 0x38, saved_z)?;
                    }
                    bytes.b11 = bytes.b11.wrapping_sub(1);
                    false
                }
                _ => false,
            }
        };
        store(&race.course, f, &bytes)?;
        if leave {
            let last = race.physics.lit_faces.borrow().count.wrapping_sub(1);
            let moved_in = listed(&race.physics, last);
            let mut lit = race.physics.lit_faces.borrow_mut();
            lit.faces[k as usize] = moved_in;
            lit.count = last;
        }
        k += 1;
        if k >= race.physics.lit_faces.borrow().count {
            return Ok(());
        }
    }
}

pub fn animate_lit_faces(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    animate_at(race, scene)
}
