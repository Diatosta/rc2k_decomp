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
//! 18 call sites in `scene::load_object`.
//! Refuses what would make the original read or write outside its tables: a file
//! elsewhere than the buffer or running past its room, more than 0x1000 points, faces
//! or normals, a point's or normal's number of 0x1000 or more, a face taken that has
//! no vertices; and a shape that does not fit the memory left (the original writes
//! past it, and `load_object` reports the fatal error afterwards).
//! Shadow mode: Rust, its writes put back, the original, compared.
//! Self-test: 60 files by lot, into memory of the test's.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::BadInput;
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::math::x87;
use crate::scene::draw_polygon::CHECKED_RVA;
use crate::scene::objects::{
    FACE_COUNT_RVA, FACE_COUNTS_RVA, FACE_CURSOR_RVA, FACE_MEMORY_RVA, FACE_ROOM_RVA, FACES,
    FACES_RVA, FILE_ROOM, FILE_RVA, FIRST_FACE_RVA, FIRST_POINT_RVA, KIND_RVA, NORMAL_COUNTS_RVA,
    POINT_COUNT_RVA, POINT_COUNTS_RVA, POINTS_ROOM_RVA, POINTS_RVA, SLOT_RVA, SLOTS,
};

// ---- game data ----

/// `dword_634FCC` faces, `dword_634FD0` points, `dword_634FD8` normals, `dword_634FE0`
/// and `dword_634FE4` the lowest and highest texture.
const COUNTS_RVA: usize = rva(0x634FCC);
/// `dword_5DDEBC` the faces', `dword_5E1EBC` the points', `dword_5E5EBC` the normals'.
const MARKS_RVA: usize = rva(0x5DDEBC);
const MARKS: usize = 0x1000;
pub const MASK_RVA: usize = rva(0x621A54); // dword_621A54
pub const PART_RVA: usize = rva(0x634FF4); // dword_634FF4
/// `dword_6350C4` the first point, `dword_6350C8` the first face, `dword_6350CC` the
/// first normal.
const FIRSTS_RVA: usize = rva(0x6350C4);
const RADIUS_RVA: usize = rva(0x5CCE14); // dword_5CCE14
const RADII_RVA: usize = rva(0x634BA0); // dword_634BA0
const DROP_RVA: usize = rva(0x613FC4); // dword_613FC4
const NORMALS_AT_RVA: usize = rva(0x5CD684); // dword_5CD684
const NO_NORMALS_RVA: usize = rva(0x631F6C); // dword_631F6C

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
    Ok(unsafe { (game_addr(FILE_RVA + offset) as *const u32).read_unaligned() })
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
        let mask: u32 = game_read(MASK_RVA);
        let slot: u32 = game_read(SLOT_RVA);
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
        if slot < 0x50 || game_read::<u32>(PART_RVA) == 0xB {
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
    unsafe {
        let room = Room {
            slot: game_read(SLOT_RVA),
            points: game_read(POINTS_RVA),
            point_count: game_read(POINT_COUNT_RVA),
            face_memory: game_read(FACE_MEMORY_RVA),
            face_cursor: game_read(FACE_CURSOR_RVA),
            face_count: game_read(FACE_COUNT_RVA),
        };
        if room.slot >= SLOTS || room.points == 0 || room.face_memory == 0 {
            return Err(BadInput(
                "a slot past the tables, or no memory for the objects",
            ));
        }
        let [points, normals, faces, face_bytes] = shape.added;
        let fits = (room.point_count as usize + points + normals) * 12
            <= game_read::<u32>(POINTS_ROOM_RVA) as usize
            && room.face_cursor as usize + face_bytes <= game_read::<u32>(FACE_ROOM_RVA) as usize
            && room.face_count as usize + faces <= FACES as usize;
        if !fits {
            return Err(BadInput("a shape that does not fit the objects' memory"));
        }
        Ok(room)
    }
}

pub unsafe fn add_object_shape(file: usize) -> Result<(), BadInput> {
    unsafe {
        if file != game_addr(FILE_RVA) {
            return Err(BadInput("an object's file elsewhere than in its buffer"));
        }
        let shape = match shape()? {
            Ok(shape) => shape,
            Err(points) => {
                game_write(COUNTS_RVA + 0x14, [u32::MAX, 0]);
                if let Some(points) = points {
                    game_write(COUNTS_RVA + 4, points);
                }
                return Ok(());
            }
        };
        let room = room(&shape)?;
        let slot = room.slot as usize * 4;
        let read = |offset: usize| (game_addr(FILE_RVA + offset) as *const u32).read_unaligned();

        game_write(COUNTS_RVA + 0x14, [u32::MAX, 0]);
        game_write(COUNTS_RVA + 4, shape.points as u32);
        game_write(COUNTS_RVA, shape.faces.len() as u32);
        game_write(COUNTS_RVA + 0xC, shape.normals as u32);
        for n in 0..MARKS {
            let taken = shape.faces.get(n).is_some_and(|face| face.taken);
            game_write(MARKS_RVA + n * 4, taken as u32);
            game_write(MARKS_RVA + (MARKS + n) * 4, shape.point_marks[n]);
            game_write(MARKS_RVA + (2 * MARKS + n) * 4, shape.normal_marks[n]);
        }

        // The points.
        let (first_point, first_face) = (room.point_count, room.face_count);
        game_write(FIRSTS_RVA, [first_point, first_face]);
        game_write(FIRST_POINT_RVA + slot, first_point);
        game_write(FIRST_FACE_RVA + slot, first_face);
        let kind: u32 = game_read(KIND_RVA);
        let turned = (0x190..0x258).contains(&kind);
        let drop: u32 = game_read(DROP_RVA);
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
            game_write(CHECKED_RVA, reach);
            if (reach.wrapping_sub(radius) as i32) >= 0 {
                radius = reach;
            }
            count += 1;
            to = to.add(1);
        }
        game_write(RADIUS_RVA, radius);
        game_write(RADII_RVA + slot, radius);
        game_write(POINT_COUNTS_RVA + slot, count - first_point);
        let first_normal = count;
        game_write(FIRSTS_RVA + 8, first_normal);

        // The faces.
        let no_normals = shape.normals == 0;
        let mut without: u32 = game_read(NO_NORMALS_RVA);
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
            game_write(FACES_RVA + face_number as usize * 4, out as u32);
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
        game_write(COUNTS_RVA + 0x14, [lowest, highest]);
        game_write(NO_NORMALS_RVA, without);
        game_write(FACE_COUNT_RVA, face_number);
        game_write(NORMALS_AT_RVA, (file + shape.normals_at) as u32);
        game_write(FACE_CURSOR_RVA, out as u32 - room.face_memory);

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
        game_write(POINT_COUNT_RVA, count);
        game_write(NORMAL_COUNTS_RVA + slot, count - first_normal);
        game_write(FACE_COUNTS_RVA + slot, face_number - first_face);
        Ok(())
    }
}
