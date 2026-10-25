//! The ground's search: which course face lies under the point (`dword_639824`,
//! `dword_63982C`, whole-number x and z), and the shares and height there (IDA,
//! 2026-10-17). `sub_441E2F` tries the last face found (`physics::try_last_face`), then
//! the nine grid cells around the point (`physics::search_cell`: the cell's course record,
//! `physics::find_course_record`, then its faces, `physics::search_record_faces`), each
//! face by `physics::face_under_point` (its edges by `physics::edge_side`; when inside,
//! `physics::face_shares` and `physics::plane_height`).
//!
//! Plain since 2026-10-23 (phase 3): the cores take what they read (the face, the record,
//! the point) and return the carry; the registers the originals leave are read by none of
//! their callers (`ground_search` keeps them all: `pusha` / `popa`).

use crate::errors::{BadInput, Refused};
use crate::framework::{game_addr, rva};
use crate::physics::edge_side::{EdgePlaces, edge_side_at};
use crate::physics::find_course_record::{RecordPlaces, find_course_record_at};
use crate::physics::ground_store::GROUND_WORK;
use crate::physics::plane_height::{Plane, plane_height_at};
use crate::race::course_store::COURSE;
use crate::race::numbers_store::NUMBERS;
use crate::scene::draw_switches_store::DRAW_SWITCHES;
use rc2k_core::physics::ground_search::{self, FaceTest};

/// The ground variables, by IDA address (the tests' copies went with the hooks).
#[derive(Clone, Copy)]
pub struct Ground;

impl Ground {
    pub fn game() -> Self {
        Self
    }

    /// The address of the variable IDA shows at `ida`.
    pub fn a(&self, ida: u32) -> u32 {
        game_addr(rva(ida as usize)) as u32
    }

    pub unsafe fn get(&self, ida: u32) -> u32 {
        unsafe { (self.a(ida) as *const u32).read_unaligned() }
    }

    pub unsafe fn put(&self, ida: u32, v: u32) {
        unsafe { (self.a(ida) as *mut u32).write_unaligned(v) }
    }

    pub fn edge(&self) -> EdgePlaces {
        EdgePlaces {
            x: GROUND_WORK.ground_x.addr() as u32,
            z: GROUND_WORK.ground_z.addr() as u32,
            product: self.a(0x620460),
        }
    }

    pub fn plane(&self) -> Plane {
        Plane {
            a: GROUND_WORK.face_corners[4].addr() as u32,
            b: GROUND_WORK.face_corners[7].addr() as u32,
            c: GROUND_WORK.face_corners[1].addr() as u32,
            f: GROUND_WORK.plane_f.addr() as u32,
            g: GROUND_WORK.plane_g.addr() as u32,
            out: GROUND_WORK.ground_height.addr() as u32,
        }
    }

    pub unsafe fn records(&self) -> RecordPlaces {
        RecordPlaces {
            firsts: self.a(0x5CC3E4),
            lasts: self.a(0x5CC7E4),
            records: COURSE.course_records.addr() as u32,
            count: COURSE.course_record_count.get(),
        }
    }

    unsafe fn point(&self, index: u32) -> u32 {
        COURSE
            .scene_points
            .get()
            .wrapping_add(index.wrapping_mul(12))
    }
}

unsafe fn at(a: u32) -> u32 {
    unsafe { (a as *const u32).read_unaligned() }
}

/// `physics::face_shares` (0x442323): the three corners named by `dword_639890` ..
/// `98` copied (0x639708 ..), the point (0x6396FC, 0x639704), the differences, and the
/// shares by the x87 (`math::x87_run::face_shares`; the `rcpss` way with the SSE switch).
/// The registers it leaves (eax the last difference, ebx, ecx the third corner's y, z)
/// are read by none of its callers.
pub unsafe fn face_shares_at(g: &Ground) {
    unsafe {
        let mut corner = [[0u32; 3]; 3];
        for (k, c) in corner.iter_mut().enumerate() {
            let p = g.point(GROUND_WORK.face_corner_numbers[k].get());
            *c = [at(p), at(p + 4), at(p + 8)];
            for (n, &v) in c.iter().enumerate() {
                GROUND_WORK.face_corners[3 * k + n].set(v);
            }
        }
        let (x, z) = (GROUND_WORK.ground_x.get(), GROUND_WORK.ground_z.get());
        GROUND_WORK.face_point_x.set(x);
        GROUND_WORK.face_point_z.set(z);
        let d = [
            (
                &GROUND_WORK.corner_x_differences[0],
                x.wrapping_sub(corner[0][0]),
            ),
            (
                &GROUND_WORK.corner_x_differences[1],
                corner[1][0].wrapping_sub(corner[0][0]),
            ),
            (
                &GROUND_WORK.corner_x_differences[2],
                corner[2][0].wrapping_sub(corner[0][0]),
            ),
            (
                &GROUND_WORK.corner_z_differences[0],
                z.wrapping_sub(corner[0][2]),
            ),
            (
                &GROUND_WORK.corner_z_differences[1],
                corner[1][2].wrapping_sub(corner[0][2]),
            ),
            (
                &GROUND_WORK.corner_z_differences[2],
                corner[2][2].wrapping_sub(corner[0][2]),
            ),
            (
                &GROUND_WORK.corner_y_differences[0],
                corner[1][1].wrapping_sub(corner[0][1]),
            ),
            (
                &GROUND_WORK.corner_y_differences[1],
                corner[2][1].wrapping_sub(corner[0][1]),
            ),
        ];
        for &(cell, value) in &d {
            cell.set(value);
        }
        let sse = DRAW_SWITCHES.use_sse.get() != 0;
        let i = [
            d[0].1,
            d[1].1,
            d[2].1,
            d[3].1,
            d[4].1,
            d[5].1,
            NUMBERS.k65536.bits(),
        ];
        let mut o = [0u32; 8];
        crate::math::x87_run::face_shares(&i, &mut o, sse);
        GROUND_WORK.face_share_a.set_bits(o[0]);
        GROUND_WORK.face_share_b.set_bits(o[1]);
        GROUND_WORK.face_share_c.set_bits(o[2]);
        GROUND_WORK.plane_f.set_bits(o[3]);
        GROUND_WORK.share_s.set(o[4]);
        GROUND_WORK.plane_g.set_bits(o[5]);
        GROUND_WORK.share_t.set(o[6]);
        if sse {
            GROUND_WORK.face_share_scale.set_bits(o[7]);
        }
    }
}

/// `face_under_point`'s refusal, worked out without writing.
pub unsafe fn face_refusal(face: u32) -> Option<&'static str> {
    ground_search::check_face(face, COURSE.course_face_count.get())
        .err()
        .map(|e| e.0)
}

/// `physics::face_under_point` (0x442006): whether the point is inside face `number`
/// (ebx; see `physics/face_under_point.rs`): the carry. The registers it leaves are read
/// by none of its callers (`search_record_faces`, `try_last_face`).
// Converted: `rc2k_core::physics::ground_search::face_under_point`.
pub unsafe fn face_under_point_at(g: &Ground, number: u32) -> Result<bool, Refused> {
    unsafe {
        if let Some(why) = face_refusal(number) {
            return Err(BadInput(why).into());
        }
        COURSE.ground_number.set(number);
        let face = (number << 6).wrapping_add(COURSE.course_faces.get());
        let kind = (face as *const u8).read();
        // One edge.
        let side = |from: u32, to: u32| -> Option<bool> {
            let s = edge_side_at(&g.edge(), g.point(at(face + from)), g.point(at(face + to)));
            (s.zero == Some(true)).then_some(s.carry)
        };
        let test = ground_search::face_under_point(
            kind,
            || ((face + 2) as *const u16).read_unaligned(),
            g.get(0x620470),
            |o| at(face + o),
            side,
        );
        let (half, corners) = match test {
            FaceTest::Kind | FaceTest::Flags(_) | FaceTest::Edge => return Ok(false),
            FaceTest::Inside { half, corners } => (half, corners),
        };
        let number = COURSE.ground_number.get();
        COURSE.ground_slot.set(number);
        let face = (number << 6).wrapping_add(COURSE.course_faces.get());
        COURSE.ground_half.set(half);
        for (k, &o) in corners.iter().enumerate() {
            let corner = at(face + o);
            GROUND_WORK.face_corner_numbers[k].set(corner);
            COURSE.ground_corners[k].set(corner);
        }
        face_shares_at(g);
        GROUND_WORK.face_under_point_done.set(1);
        plane_height_at(&g.plane());
        Ok(true)
    }
}

/// `search_record_faces`' refusal, worked out without writing.
pub unsafe fn record_refusal(record: u32) -> Option<&'static str> {
    unsafe {
        if let Err(e) = ground_search::check_record(record, COURSE.course_record_count.get()) {
            return Some(e.0);
        }
        let rec = (record << 5).wrapping_add(COURSE.course_records.get());
        ground_search::check_record_faces(
            at(rec + 8),
            at(rec + 0xC),
            COURSE.course_face_count.get(),
        )
        .err()
        .map(|e| e.0)
    }
}

/// `physics::search_record_faces` (0x441FD9): faces +8 ..= +0xC of course record
/// `record` (ebx), each by `face_under_point`; the carry when one has the point.
// Converted: `rc2k_core::physics::ground_search::search_faces`.
pub unsafe fn search_record_faces_at(g: &Ground, record: u32) -> Result<bool, Refused> {
    unsafe {
        if let Some(why) = record_refusal(record) {
            return Err(BadInput(why).into());
        }
        let rec = (record << 5).wrapping_add(COURSE.course_records.get());
        let (found, _) = ground_search::search_faces(at(rec + 8), at(rec + 0xC), |face| {
            face_under_point_at(g, face)
        })?;
        Ok(found)
    }
}

/// `physics::search_cell` (0x441FBA): the cell of (`x`, `z`) (eax, ecx; each >> 21; key
/// z << 11 + x), its record by `find_course_record`, then its faces. The carry when
/// found.
// Converted: `rc2k_core::physics::ground_search::cell_key`.
pub unsafe fn search_cell_at(g: &Ground, x: u32, z: u32) -> Result<bool, Refused> {
    unsafe {
        let (key, _) = ground_search::cell_key(x, z);
        let Some(record) = find_course_record_at(&g.records(), key)? else {
            return Ok(false);
        };
        if let Some(why) = record_refusal(record) {
            return Err(BadInput(why).into());
        }
        search_record_faces_at(g, record)
    }
}

/// `physics::try_last_face` (0x441E0C): ebx = `dword_5CCC78`; when it is a face (not -1,
/// below the count) `face_under_point` on it inside `pusha`/`popa`. The carry when found.
// Converted: `rc2k_core::physics::ground_search::last_face`.
pub unsafe fn try_last_face_at(g: &Ground) -> Result<bool, Refused> {
    unsafe {
        let last = COURSE.ground_face_last.get();
        if ground_search::last_face(last, COURSE.course_face_count.get()).is_none() {
            return Ok(false);
        }
        face_under_point_at(g, last)
    }
}

pub use rc2k_core::physics::ground_search::CELLS;

/// `physics::ground_search` (0x441E2F), inside `pusha` / `popa`: `try_last_face`; else
/// `dword_5CCC74` = -1 and `search_cell` on the point plus each of `CELLS` until one
/// finds it. `g_groundFound` 1 when found, else 0. Registers kept; the carry the found.
// Converted: `rc2k_core::physics::ground_search::ground_search`.
pub unsafe fn ground_search_at(g: &Ground) -> Result<bool, Refused> {
    unsafe {
        if let Some(why) = ground_refusal(g) {
            return Err(BadInput(why).into());
        }
        let found = ground_search::ground_search(
            || try_last_face_at(g),
            || COURSE.ground_slot.set(ground_search::NO_FACE),
            || (GROUND_WORK.ground_x.get(), GROUND_WORK.ground_z.get()),
            |x, z| search_cell_at(g, x, z),
        )?;
        GROUND_WORK.ground_found.set(u32::from(found));
        Ok(found)
    }
}

/// `ground_search`'s refusals, worked out without writing: those of the cells' records it
/// would come to (all nine are looked at; a refusal past the face found is refused too).
pub unsafe fn ground_refusal(g: &Ground) -> Option<&'static str> {
    unsafe {
        let records = g.records();
        for (dx, dz) in CELLS {
            let x = GROUND_WORK.ground_x.get().wrapping_add(dx);
            let z = GROUND_WORK.ground_z.get().wrapping_add(dz);
            match find_course_record_at(&records, ground_search::cell_key(x, z).0) {
                Err(_) => return Some("a cell's kind runs past g_courseRecordCount"),
                Ok(Some(record)) => {
                    if let Some(why) = record_refusal(record) {
                        return Some(why);
                    }
                }
                Ok(None) => {}
            }
        }
        None
    }
}
