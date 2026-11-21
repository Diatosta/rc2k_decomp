//! `scene::add_object_shape`: puts a parsed shape into the objects' tables as the slot being
//! loaded.
//!
//! The shape's points are added to the points' memory from the object point count on, its
//! faces to the faces' memory from the face cursor on (points and normals renumbered from the
//! slot's first ones), its normals after the points; the slot's first point and face, its
//! counts, its radius and the marks tables are set. A file of no points or no faces only sets
//! the counts of the file. Refused: a slot past the tables, no memory for the objects, and a
//! shape that does not fit the memory left (`load_object` reports the fatal errors).

use crate::errors::BadInput;
use crate::scene::objects::{FACES, SLOTS};
use crate::scene::owner::Scene;
use rc2k_core::scene::object_shape::{MARKS, ObjectShape, Parsed};

/// Where the shape goes, checked for room.
struct Room {
    points: u32,
    point_count: u32,
    face_memory: u32,
    face_cursor: u32,
    face_count: u32,
}

fn room(
    course: &crate::race::owner_course::Course,
    scene: &Scene,
    slot: u32,
    shape: &ObjectShape,
) -> Result<Room, BadInput> {
    let room = Room {
        points: course.lists.get().object_points.get(),
        point_count: scene.objects.tables.borrow().point_count,
        face_memory: course.lists.get().face_memory.get(),
        face_cursor: scene.object_work().face_cursor,
        face_count: scene.texture_levels().face_count,
    };
    if slot >= SLOTS || room.points == 0 || room.face_memory == 0 {
        return Err(BadInput(
            "a slot past the tables, or no memory for the objects",
        ));
    }
    let added = shape.added();
    let fits = (room.point_count as usize + added.points + added.normals) * 12
        <= course.lists.get().object_points_room.get() as usize
        && room.face_cursor as usize + added.face_bytes
            <= course.lists.get().face_memory_room.get() as usize
        && room.face_count as usize + added.faces <= FACES as usize;
    if !fits {
        return Err(BadInput("a shape that does not fit the objects' memory"));
    }
    Ok(room)
}

/// Stores `parsed` as slot `slot`.
pub fn add_object_shape(
    race: &crate::race::owner::Race,
    scene: &Scene,
    slot: u32,
    parsed: &Parsed,
) -> Result<(), BadInput> {
    let course = &race.course;
    scene.object_work_mut().loading_slot = slot;
    scene.object_work_mut().shape_counts[5..7].copy_from_slice(&[u32::MAX, 0]);
    let shape = match parsed {
        Parsed::Shape(shape) => shape,
        Parsed::Empty { points } => {
            if let Some(points) = points {
                scene.object_work_mut().shape_counts[1] = *points;
            }
            return Ok(());
        }
    };
    let room = room(course, scene, slot, shape)?;
    let slot = slot as usize;
    scene.object_work_mut().shape_counts[1] = shape.file_points;
    scene.object_work_mut().shape_counts[0] = shape.file_faces;
    scene.object_work_mut().shape_counts[3] = shape.file_normals;
    let mut places = scene.texture_places_mut();
    for n in 0..MARKS {
        places.face_marks[n] = shape.face_taken[n] as u32;
        places.point_marks[n] = shape.point_marks[n];
        places.normal_marks[n] = shape.normal_marks[n];
    }
    drop(places);

    // The points.
    let (first_point, first_face) = (room.point_count, room.face_count);
    scene.object_work_mut().shape_firsts[..2].copy_from_slice(&[first_point, first_face]);
    scene.objects.slots.borrow_mut().first_point[slot] = first_point;
    scene.objects.slots.borrow_mut().first_face[slot] = first_face;
    let mut count = first_point;
    let mut to = room.points.wrapping_add(count.wrapping_mul(12));
    for point in &shape.points {
        race.course.stage.object_points.set_point(to, *point)?;
        count += 1;
        to = to.wrapping_add(12);
    }
    if let Some(reach) = shape.last_reach {
        scene.draw_work_mut().scratch = reach;
    }
    scene.loaded_slots_mut().radii[slot] = shape.radius;
    scene.objects.slots.borrow_mut().point_counts[slot] = count - first_point;
    let first_normal = count;
    scene.object_work_mut().shape_firsts[2] = first_normal;

    // The faces.
    let mut face_number = first_face;
    let mut out = (room.face_memory + room.face_cursor) as usize;
    for n in 0..shape.faces.len() {
        crate::scene::objects::face_cell(race.faces(), face_number as usize).set(out as u32);
        let bytes = shape.face_bytes(n, first_point, first_normal);
        race.course.stage.object_faces.write(out as u32, &bytes)?;
        out += bytes.len();
        face_number += 1;
    }
    scene.object_work_mut().shape_counts[5..7]
        .copy_from_slice(&[shape.lowest_texture, shape.highest_texture]);
    {
        let mut slots = scene.loaded_slots_mut();
        slots.no_normals = slots.no_normals.wrapping_add(shape.missing_normals);
    }
    scene.texture_levels_mut().face_count = face_number;
    scene.object_work_mut().face_cursor = out as u32 - room.face_memory;

    // The normals.
    for normal in &shape.normals {
        race.course.stage.object_points.set_point(to, *normal)?;
        count += 1;
        to = to.wrapping_add(12);
    }
    scene.objects.tables.borrow_mut().point_count = count;
    scene.objects.slots.borrow_mut().normal_counts[slot] = count - first_normal;
    scene.objects.slots.borrow_mut().face_counts[slot] = face_number - first_face;
    Ok(())
}
