//! `scene::add_object_shape` (0x426695): takes the faces of the file just read that
//! belong to the part being loaded, with their points and normals, into the objects'
//! tables as the slot `dword_634FEC`.
//!
//! In:  esi = the file (`byte_A2BB54`)
//! Out: the original leaves every register changed; its caller reads none. The Rust
//!      version keeps them.
//!
//! The file: +4 the number of points, +8 of faces, +0xC of normals; from +0x2C the
//! points (three whole numbers each); then the faces, each 0x10 bytes (+0 its
//! vertices' number, +4 its texture as a word, +7 its group in the low five bits,
//! +0xC its normal) and 0x10 bytes a vertex (+0 its point, +4 and +8 texture
//! coordinates, +0xC its normal); then the normals, twelve bytes each.
//!
//! - `dword_634FE0` = -1 and `dword_634FE4` = 0 (the lowest and highest texture met);
//!   a file of no points or no faces ends it there.
//! - A face is taken if bit (its group) of the mask `dword_621A54` is set. The points
//!   and normals of the faces taken are marked; for a slot under 0x50, and for part
//!   0xB of a car (`dword_634FF4`), every point is. The marks become the numbers the
//!   points and normals get, in their order (`dword_5E1EBC`, `dword_5E5EBC`; -1 = not
//!   taken; the faces' marks in `dword_5DDEBC`).
//! - The marked points are added to the points' memory from `g_objectPointCount` on.
//!   For kinds 0x190 .. 0x257 a point (x, y, z) becomes (z, y - `dword_613FC4`, -x).
//!   The largest distance of a point from the y axis (`fsqrt` of x squared plus z
//!   squared, as a whole number) is the slot's radius (`dword_634BA0`).
//! - The faces taken are copied to the faces' memory, each vertex's point and every
//!   normal renumbered (a file without normals gives normal 0 and counts
//!   `dword_631F6C` up), and listed in `g_faces`.
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
use crate::physics::tables_store::PHYSICS;
use crate::race::course_lists_store::COURSE_LISTS;
use crate::race::course_store::COURSE;
use crate::scene::draw_store::DRAW;
use crate::scene::draw_switches_store::DRAW_SWITCHES;
use crate::scene::object_slots_store::OBJECT_SLOTS;
use crate::scene::object_store::OBJECT_WORK;
use crate::scene::object_tables_store::OBJECT_TABLES;
use crate::scene::objects::{FACES, FILE_ROOM, SLOTS};
use crate::scene::texture_places_store::TEXTURE_PLACES;

// ---- game data ----

/// `face_marks` (`dword_5DDEBC`) the faces', `point_marks` (`dword_5E1EBC`) the points',
/// `normal_marks` (`dword_5E5EBC`) the normals'.
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

unsafe fn dword(offset: usize) -> Result<u32, BadInput> {
    if offset.checked_add(4).is_none_or(|end| end > FILE_ROOM) {
        return Err(PAST);
    }
    Ok(unsafe { ((crate::scene::objects::file_addr() + offset) as *const u32).read_unaligned() })
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
unsafe fn shape() -> Result<Result<Shape, Option<u32>>, BadInput> {
    unsafe {
        let points = dword(4)? as usize;
        if points == 0 {
            return Ok(Err(None));
        }
        let face_count = dword(8)? as usize;
        if face_count == 0 {
            return Ok(Err(Some(points as u32)));
        }
        let normals = dword(0xC)? as usize;
        if points > MARKS || face_count > MARKS || normals > MARKS {
            return Err(BadInput("more points, faces or normals than can be marked"));
        }
        let mask: u32 = DRAW_SWITCHES.loading_group_mask.bits();
        let slot: u32 = OBJECT_WORK.loading_slot.bits();
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
            let vertices = dword(at)? as usize;
            let end = vertices
                .checked_mul(0x10)
                .and_then(|bytes| bytes.checked_add(at + 0x10))
                .filter(|&end| end <= FILE_ROOM)
                .ok_or(PAST)?;
            let taken = mask & (1 << ((dword(at + 4)? >> 24) & 0x1F)) != 0;
            if taken {
                if vertices == 0 {
                    return Err(BadInput("a face taken that has no vertices"));
                }
                marked(&mut normal_marks, dword(at + 0xC)?)?;
                for vertex in 0..vertices {
                    let v = at + 0x10 + vertex * 0x10;
                    marked(&mut normal_marks, dword(v + 0xC)?)?;
                    // Marked by its low word, looked up by the whole dword.
                    marked(&mut point_marks, dword(v)?)?;
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
        if slot < 0x50 || OBJECT_WORK.loading_car[1].get() == 0xB {
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

unsafe fn room(shape: &Shape) -> Result<Room, BadInput> {
    let room = Room {
        slot: OBJECT_WORK.loading_slot.bits(),
        points: COURSE_LISTS.object_points.bits(),
        point_count: OBJECT_TABLES.point_count.get(),
        face_memory: COURSE_LISTS.face_memory.bits(),
        face_cursor: OBJECT_WORK.face_cursor.bits(),
        face_count: OBJECT_TABLES.face_count.get(),
    };
    if room.slot >= SLOTS || room.points == 0 || room.face_memory == 0 {
        return Err(BadInput(
            "a slot past the tables, or no memory for the objects",
        ));
    }
    let [points, normals, faces, face_bytes] = shape.added;
    let fits = (room.point_count as usize + points + normals) * 12
        <= COURSE_LISTS.object_points_room.bits() as usize
        && room.face_cursor as usize + face_bytes <= COURSE_LISTS.face_memory_room.bits() as usize
        && room.face_count as usize + faces <= FACES as usize;
    if !fits {
        return Err(BadInput("a shape that does not fit the objects' memory"));
    }
    Ok(room)
}

pub unsafe fn add_object_shape(file: usize) -> Result<(), BadInput> {
    unsafe {
        if file != crate::scene::objects::file_addr() {
            return Err(BadInput("an object's file elsewhere than in its buffer"));
        }
        let shape = match shape()? {
            Ok(shape) => shape,
            Err(points) => {
                crate::session::write_first(&OBJECT_WORK.shape_counts[5..7], &[u32::MAX, 0]);
                if let Some(points) = points {
                    OBJECT_WORK.shape_counts[1].set_bits(points);
                }
                return Ok(());
            }
        };
        let room = room(&shape)?;
        let slot = room.slot as usize;
        let read = |offset: usize| {
            ((crate::scene::objects::file_addr() + offset) as *const u32).read_unaligned()
        };

        crate::session::write_first(&OBJECT_WORK.shape_counts[5..7], &[u32::MAX, 0]);
        OBJECT_WORK.shape_counts[1].set_bits(shape.points as u32);
        OBJECT_WORK.shape_counts[0].set_bits(shape.faces.len() as u32);
        OBJECT_WORK.shape_counts[3].set_bits(shape.normals as u32);
        for n in 0..MARKS {
            let taken = shape.faces.get(n).is_some_and(|face| face.taken);
            TEXTURE_PLACES.face_marks[n].set(taken as u32);
            TEXTURE_PLACES.point_marks[n].set(shape.point_marks[n]);
            TEXTURE_PLACES.normal_marks[n].set(shape.normal_marks[n]);
        }

        // The points.
        let (first_point, first_face) = (room.point_count, room.face_count);
        crate::session::write_prefix(&OBJECT_WORK.shape_firsts, &[first_point, first_face]);
        OBJECT_SLOTS.first_point[slot].set(first_point);
        OBJECT_SLOTS.first_face[slot].set(first_face);
        let kind: u32 = OBJECT_WORK.loading_kind.bits();
        let turned = (0x190..0x258).contains(&kind);
        let drop: u32 = PHYSICS.ground_drop.bits();
        let mut radius = 0u32;
        let mut count = first_point;
        let mut to = (room.points as usize + count as usize * 12) as *mut [u32; 3];
        for n in 0..shape.points {
            if shape.point_marks[n] == u32::MAX {
                continue;
            }
            let at = 0x2C + n * 12;
            let mut point = [read(at), read(at + 4), read(at + 8)];
            if turned {
                point = [
                    point[2],
                    point[1].wrapping_sub(drop),
                    point[0].wrapping_neg(),
                ];
            }
            to.write_unaligned(point);
            let reach = x87::int_hypot(point[0] as i32, point[2] as i32) as u32;
            DRAW.scratch.set_bits(reach);
            if (reach.wrapping_sub(radius) as i32) >= 0 {
                radius = reach;
            }
            count += 1;
            to = to.add(1);
        }
        COURSE.shape_radius.set_bits(radius);
        OBJECT_SLOTS.radii[slot].set(radius);
        OBJECT_SLOTS.point_counts[slot].set(count - first_point);
        let first_normal = count;
        OBJECT_WORK.shape_firsts[2].set_bits(first_normal);

        // The faces.
        let no_normals = shape.normals == 0;
        let mut without: u32 = OBJECT_SLOTS.no_normals.get();
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
            crate::scene::objects::face_cell(face_number as usize).set(out as u32);
            let put = |at: usize, value: u32| (at as *mut u32).write_unaligned(value);
            put(out, read(face.at));
            put(out + 4, read(face.at + 4));
            put(out + 8, read(face.at + 8));
            let texture = read(face.at + 4) & 0xFFFF;
            if texture <= lowest {
                lowest = texture;
            }
            if texture >= highest {
                highest = texture;
            }
            put(out + 0xC, normal(read(face.at + 0xC)));
            for vertex in 0..face.vertices {
                let (from, to) = (face.at + 0x10 + vertex * 0x10, out + 0x10 + vertex * 0x10);
                put(
                    to,
                    shape.point_marks[read(from) as usize].wrapping_add(first_point),
                );
                put(to + 4, read(from + 4));
                put(to + 8, read(from + 8));
                put(to + 0xC, normal(read(from + 0xC)));
            }
            out += 0x10 + face.vertices * 0x10;
            face_number += 1;
        }
        crate::session::write_first(&OBJECT_WORK.shape_counts[5..7], &[lowest, highest]);
        OBJECT_SLOTS.no_normals.set(without);
        OBJECT_TABLES.face_count.set(face_number);
        OBJECT_TABLES
            .normals_at
            .set((file + shape.normals_at) as u32);
        OBJECT_WORK
            .face_cursor
            .set_bits(out as u32 - room.face_memory);

        // The normals.
        for n in 0..shape.normals {
            if shape.normal_marks[n] == u32::MAX {
                continue;
            }
            let at = shape.normals_at + n * 12;
            to.write_unaligned([read(at), read(at + 4), read(at + 8)]);
            count += 1;
            to = to.add(1);
        }
        OBJECT_TABLES.point_count.set(count);
        OBJECT_SLOTS.normal_counts[slot].set(count - first_normal);
        OBJECT_SLOTS.face_counts[slot].set(face_number - first_face);
        Ok(())
    }
}
