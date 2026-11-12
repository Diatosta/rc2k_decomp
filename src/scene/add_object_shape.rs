//! `scene::add_object_shape` (0x426695): takes the faces of the file just read that
//! belong to the part being loaded, with their points and normals, into the objects'
//! tables as the slot being loaded (`loading_slot`).
//!
//! Takes: the file just read (the object file room)
//!
//! The file: +4 the number of points, +8 of faces, +0xC of normals; from +0x2C the
//! points (three whole numbers each); then the faces, each 0x10 bytes (+0 its
//! vertices' number, +4 its texture as a word, +7 its group in the low five bits,
//! +0xC its normal) and 0x10 bytes a vertex (+0 its point, +4 and +8 texture
//! coordinates, +0xC its normal); then the normals, twelve bytes each.
//!
//! - The lowest texture met is set to -1 and the highest to 0 (two cells of `shape_counts`);
//!   a file of no points or no faces ends it there.
//! - A face is taken if bit (its group) of the `loading_group_mask` is set. The points
//!   and normals of the faces taken are marked; for a slot under 0x50, and for part
//!   0xB of a car (`loading_car[1]`), every point is. The marks become the numbers the
//!   points and normals get, in their order (-1 = not taken; the faces have marks too).
//! - The marked points are added to the points' memory from the object point count on.
//!   For kinds 0x190 .. 0x257 a point (x, y, z) becomes (z, y - `ground_drop`, -x).
//!   The largest distance of a point from the y axis (`fsqrt` of x squared plus z
//!   squared, as a whole number) is the slot's radius (`radii`).
//! - The faces taken are copied to the faces' memory, each vertex's point and every
//!   normal renumbered (a file without normals gives normal 0 and counts
//!   `no_normals` up), and listed in the faces table.
//! - The marked normals are added after the points.
//! - The slot's first point and face and its numbers of points, normals and faces are
//!   set.
//!
//! 18 call sites in `scene::load_object`. Refuses what would make the original read or write
//! outside its tables: a file elsewhere than the buffer or running past its room, more than
//! 0x1000 points, faces or normals, a point's or normal's number of 0x1000 or more, a face
//! taken that has no vertices; and a shape that does not fit the memory left (the original
//! writes past it, and `load_object` reports the fatal error afterwards).

use crate::errors::BadInput;
use crate::math::x87;
use crate::scene::objects::{FACES, FILE_ROOM, SLOTS};
use crate::scene::owner::Scene;

// ---- game data ----

/// Entries in the marks tables: `face_marks` the faces', `point_marks` the points',
/// `normal_marks` the normals'.
const MARKS: usize = 0x1000;

// ---- Rust version ----

/// A face of the file: where it is, its vertices, whether it is taken.
struct Face {
    at: usize,
    vertices: usize,
    taken: bool,
}

/// What the file gives, worked out without writing.
struct Shape {
    points: usize,
    normals: usize,
    faces: Vec<Face>,
    /// Where the normals start in the file.
    normals_at: usize,
    /// -1, or the number each point and normal gets.
    point_marks: Vec<u32>,
    normal_marks: Vec<u32>,
    /// How much is added: points, normals, faces, bytes of faces.
    added: [usize; 4],
}

const PAST: BadInput = BadInput("the object's file runs past its room");

fn dword(race: &crate::race::owner::Race, offset: usize) -> Result<u32, BadInput> {
    if offset.checked_add(4).is_none_or(|end| end > FILE_ROOM) {
        return Err(PAST);
    }
    Ok(u32::from_le_bytes(
        crate::scene::objects::file(race)
            .read_at(offset, 4, "a word of an object's file")?
            .try_into()
            .unwrap(),
    ))
}

fn numbered(marks: &mut [u32]) {
    let mut next = 0;
    for mark in marks {
        if *mark == 0 {
            *mark = u32::MAX;
        } else {
            *mark = next;
            next += 1;
        }
    }
}

/// None: a file of no points or no faces (the second with the points' number).
fn shape(
    race: &crate::race::owner::Race,
    scene: &Scene,
) -> Result<Result<Shape, Option<u32>>, BadInput> {
    let points = dword(race, 4)? as usize;
    if points == 0 {
        return Ok(Err(None));
    }
    let face_count = dword(race, 8)? as usize;
    if face_count == 0 {
        return Ok(Err(Some(points as u32)));
    }
    let normals = dword(race, 0xC)? as usize;
    if points > MARKS || face_count > MARKS || normals > MARKS {
        return Err(BadInput("more points, faces or normals than can be marked"));
    }
    let mask: u32 = scene.draw_switches.loading_group_mask.get();
    let slot: u32 = scene.object_work().loading_slot.get();
    let mut point_marks = vec![0u32; MARKS];
    let mut normal_marks = vec![0u32; MARKS];
    let mut faces = Vec::with_capacity(face_count);
    let mut at = 0x2C + points * 12;
    let mut added = [0usize; 4];
    let marked = |marks: &mut Vec<u32>, number: u32| {
        let mark = marks
            .get_mut(number as usize)
            .ok_or(BadInput("a point's or normal's number of 0x1000 or more"))?;
        *mark = 1;
        Ok::<(), BadInput>(())
    };
    for _ in 0..face_count {
        let vertices = dword(race, at)? as usize;
        let end = vertices
            .checked_mul(0x10)
            .and_then(|bytes| bytes.checked_add(at + 0x10))
            .filter(|&end| end <= FILE_ROOM)
            .ok_or(PAST)?;
        let taken = mask & (1 << ((dword(race, at + 4)? >> 24) & 0x1F)) != 0;
        if taken {
            if vertices == 0 {
                return Err(BadInput("a face taken that has no vertices"));
            }
            marked(&mut normal_marks, dword(race, at + 0xC)?)?;
            for vertex in 0..vertices {
                let v = at + 0x10 + vertex * 0x10;
                marked(&mut normal_marks, dword(race, v + 0xC)?)?;
                // Marked by its low word, looked up by the whole dword.
                marked(&mut point_marks, dword(race, v)?)?;
            }
            added[2] += 1;
            added[3] += end - at;
        }
        faces.push(Face {
            at,
            vertices,
            taken,
        });
        at = end;
    }
    if at + normals * 12 > FILE_ROOM {
        return Err(PAST);
    }
    if slot < 0x50 || scene.object_work().loading_car[1].get() == 0xB {
        point_marks[..points].fill(1);
    }
    numbered(&mut point_marks);
    numbered(&mut normal_marks);
    added[0] = point_marks[..points]
        .iter()
        .filter(|&&m| m != u32::MAX)
        .count();
    added[1] = normal_marks[..normals]
        .iter()
        .filter(|&&m| m != u32::MAX)
        .count();
    Ok(Ok(Shape {
        points,
        normals,
        faces,
        normals_at: at,
        point_marks,
        normal_marks,
        added,
    }))
}

/// Where the shape goes, checked for room.
struct Room {
    slot: u32,
    points: u32,
    point_count: u32,
    face_memory: u32,
    face_cursor: u32,
    face_count: u32,
}

fn room(
    course: &crate::race::owner_course::Course,
    scene: &Scene,
    shape: &Shape,
) -> Result<Room, BadInput> {
    let room = Room {
        slot: scene.object_work().loading_slot.get(),
        points: course.lists.object_points.get(),
        point_count: scene.objects.tables.point_count.get(),
        face_memory: course.lists.face_memory.get(),
        face_cursor: scene.object_work().face_cursor.get(),
        face_count: scene.objects.tables.face_count.get(),
    };
    if room.slot >= SLOTS || room.points == 0 || room.face_memory == 0 {
        return Err(BadInput(
            "a slot past the tables, or no memory for the objects",
        ));
    }
    let [points, normals, faces, face_bytes] = shape.added;
    let fits = (room.point_count as usize + points + normals) * 12
        <= course.lists.object_points_room.get() as usize
        && room.face_cursor as usize + face_bytes <= course.lists.face_memory_room.get() as usize
        && room.face_count as usize + faces <= FACES as usize;
    if !fits {
        return Err(BadInput("a shape that does not fit the objects' memory"));
    }
    Ok(room)
}

pub fn add_object_shape(
    race: &crate::race::owner::Race,
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    let course = &race.course;
    let shape = match shape(race, scene)? {
        Ok(shape) => shape,
        Err(points) => {
            crate::session::write_first(&scene.object_work().shape_counts[5..7], &[u32::MAX, 0]);
            if let Some(points) = points {
                scene.object_work().shape_counts[1].set(points);
            }
            return Ok(());
        }
    };
    let room = room(course, scene, &shape)?;
    let slot = room.slot as usize;
    let read = |offset: usize| -> Result<u32, BadInput> {
        Ok(u32::from_le_bytes(
            crate::scene::objects::file(race)
                .read_at(offset, 4, "a word of an object's file")?
                .try_into()
                .unwrap(),
        ))
    };

    crate::session::write_first(&scene.object_work().shape_counts[5..7], &[u32::MAX, 0]);
    scene.object_work().shape_counts[1].set(shape.points as u32);
    scene.object_work().shape_counts[0].set(shape.faces.len() as u32);
    scene.object_work().shape_counts[3].set(shape.normals as u32);
    for n in 0..MARKS {
        let taken = shape.faces.get(n).is_some_and(|face| face.taken);
        scene.textures.places.face_marks[n].set(taken as u32);
        scene.textures.places.point_marks[n].set(shape.point_marks[n]);
        scene.textures.places.normal_marks[n].set(shape.normal_marks[n]);
    }

    // The points.
    let (first_point, first_face) = (room.point_count, room.face_count);
    crate::session::write_prefix(
        &scene.object_work().shape_firsts,
        &[first_point, first_face],
    );
    scene.objects.slots.first_point[slot].set(first_point);
    scene.objects.slots.first_face[slot].set(first_face);
    let kind: u32 = scene.object_work().loading_kind.get();
    let turned = (0x190..0x258).contains(&kind);
    let drop: u32 = race.shared.physics_tables.ground_drop.get();
    let mut radius = 0u32;
    let mut count = first_point;
    let mut to = room.points.wrapping_add(count.wrapping_mul(12));
    for n in 0..shape.points {
        if shape.point_marks[n] == u32::MAX {
            continue;
        }
        let at = 0x2C + n * 12;
        let mut point = [read(at)?, read(at + 4)?, read(at + 8)?];
        if turned {
            point = [
                point[2],
                point[1].wrapping_sub(drop),
                point[0].wrapping_neg(),
            ];
        }
        crate::scene::object_points::set_point(to, point)?;
        let reach = x87::int_hypot(point[0] as i32, point[2] as i32) as u32;
        scene.drawing.scene.scratch.set(reach);
        if (reach.wrapping_sub(radius) as i32) >= 0 {
            radius = reach;
        }
        count += 1;
        to = to.wrapping_add(12);
    }
    course.course.shape_radius.set(radius);
    scene.objects.slots.radii[slot].set(radius);
    scene.objects.slots.point_counts[slot].set(count - first_point);
    let first_normal = count;
    scene.object_work().shape_firsts[2].set(first_normal);

    // The faces.
    let no_normals = shape.normals == 0;
    let mut without: u32 = scene.objects.slots.no_normals.get();
    let (mut lowest, mut highest) = (u32::MAX, 0u32);
    let mut face_number = first_face;
    let mut out = (room.face_memory + room.face_cursor) as usize;
    let mut normal = |number: u32| {
        if no_normals {
            without = without.wrapping_add(1);
            0
        } else {
            shape.normal_marks[number as usize].wrapping_add(first_normal)
        }
    };
    for face in shape.faces.iter().filter(|face| face.taken) {
        crate::scene::objects::face_cell(race.faces(), face_number as usize).set(out as u32);
        let len = 0x10 + face.vertices * 0x10;
        let source = crate::scene::objects::file(race).read_at(
            face.at,
            len,
            "a face of an object's file",
        )?;
        let word = |at: usize| u32::from_le_bytes(source[at..at + 4].try_into().unwrap());
        let mut made = Vec::with_capacity(len);
        let mut push = |value: u32| made.extend_from_slice(&value.to_le_bytes());
        push(word(0));
        push(word(4));
        push(word(8));
        let texture = word(4) & 0xFFFF;
        if texture <= lowest {
            lowest = texture;
        }
        if texture >= highest {
            highest = texture;
        }
        push(normal(word(0xC)));
        for vertex in 0..face.vertices {
            let from = 0x10 + vertex * 0x10;
            let mark = *shape.point_marks.get(word(from) as usize).ok_or(PAST)?;
            push(mark.wrapping_add(first_point));
            push(word(from + 4));
            push(word(from + 8));
            push(normal(word(from + 0xC)));
        }
        crate::scene::object_faces::write(out as u32, &made)?;
        out += 0x10 + face.vertices * 0x10;
        face_number += 1;
    }
    crate::session::write_first(&scene.object_work().shape_counts[5..7], &[lowest, highest]);
    scene.objects.slots.no_normals.set(without);
    scene.objects.tables.face_count.set(face_number);
    scene
        .object_work()
        .face_cursor
        .set(out as u32 - room.face_memory);

    // The normals.
    for n in 0..shape.normals {
        if shape.normal_marks[n] == u32::MAX {
            continue;
        }
        let at = shape.normals_at + n * 12;
        crate::scene::object_points::set_point(to, [read(at)?, read(at + 4)?, read(at + 8)?])?;
        count += 1;
        to = to.wrapping_add(12);
    }
    scene.objects.tables.point_count.set(count);
    scene.objects.slots.normal_counts[slot].set(count - first_normal);
    scene.objects.slots.face_counts[slot].set(face_number - first_face);
    Ok(())
}
