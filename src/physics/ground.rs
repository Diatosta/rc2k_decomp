//! The ground's search: which course face lies under the point (`ground_x`,
//! `ground_z`, whole-number x and z), and the shares and height there. `physics::ground_search` tries the last face found (`physics::try_last_face`), then
//! the nine grid cells around the point (`physics::search_cell`: the cell's course record,
//! `physics::find_course_record`, then its faces, `physics::search_record_faces`), each
//! face by `physics::face_under_point` (its edges by `physics::edge_side`; when inside,
//! `physics::face_shares` and `physics::plane_height`).
//!
//! The cores take what they read (the face, the record, the point) and return whether the
//! point was found.

use crate::errors::{BadInput, Refused};
use crate::physics::cells::Trap;
use crate::physics::edge_side::edge_side;
use crate::physics::find_course_record::find_course_record;
use crate::physics::ground_query::GroundQuery;
use crate::physics::plane_height::plane_height;
use crate::race::owner_course::CourseReads;
use rc2k_core::physics::ground_search::{self, FaceTest};

/// The first and last faces (+8, +0xC) of course record `record`.
fn record_faces(course: CourseReads<'_>, record: u32) -> Result<(u32, u32), BadInput> {
    let face = |offset| crate::race::course_places::record_field::<u32>(course, record, offset);
    Ok((face(8)?, face(0xC)?))
}

/// `physics::face_shares` (0x442323): the three corners named by `face_corner_numbers` .. `98` copied
/// (0x639708 ..), the point (0x6396FC, 0x639704), the differences, and the shares by the x87
/// (`math::x87::face_shares`; the `rcpss` way with the SSE switch).
pub fn face_shares_at(
    course: CourseReads<'_>,
    q: &GroundQuery<'_>,
    switches: &crate::scene::draw_switches_store::DrawSwitches,
) -> Result<(), Refused> {
    let mut corner = [[0u32; 3]; 3];
    for (k, c) in corner.iter_mut().enumerate() {
        *c = crate::race::scene_points::point_number(course, q.points, q.corner_numbers[k].get())?;
        for (n, &v) in c.iter().enumerate() {
            q.corners[3 * k + n].set(v);
        }
    }
    let (x, z) = (q.x.get(), q.z.get());
    q.point_x.set(x);
    q.point_z.set(z);
    let d = [
        (&q.x_differences[0], x.wrapping_sub(corner[0][0])),
        (&q.x_differences[1], corner[1][0].wrapping_sub(corner[0][0])),
        (&q.x_differences[2], corner[2][0].wrapping_sub(corner[0][0])),
        (&q.z_differences[0], z.wrapping_sub(corner[0][2])),
        (&q.z_differences[1], corner[1][2].wrapping_sub(corner[0][2])),
        (&q.z_differences[2], corner[2][2].wrapping_sub(corner[0][2])),
        (&q.y_differences[0], corner[1][1].wrapping_sub(corner[0][1])),
        (&q.y_differences[1], corner[2][1].wrapping_sub(corner[0][1])),
    ];
    for &(cell, value) in &d {
        cell.set(value);
    }
    let sse = switches.use_sse.get() != 0;
    let shares = crate::math::x87::face_shares(
        &crate::math::x87::FaceDifferences {
            x: [d[0].1 as i32, d[1].1 as i32, d[2].1 as i32],
            z: [d[3].1 as i32, d[4].1 as i32, d[5].1 as i32],
            scale: &course.numbers.k65536,
        },
        sse,
    );
    q.share_a.set(shares.share_a);
    q.share_b.set(shares.share_b);
    q.share_c.set(shares.share_c);
    q.plane_f.set(shares.plane_f);
    q.share_s.set(shares.share_s as u32);
    q.plane_g.set(shares.plane_g);
    q.share_t.set(shares.share_t as u32);
    if let Some(scale) = shares.share_scale {
        q.share_scale.set(scale);
    }
    Ok(())
}

/// `face_under_point`'s refusal, worked out without writing.
pub fn face_refusal(course: CourseReads<'_>, face: u32) -> Option<&'static str> {
    ground_search::check_face(face, course.course.course_face_count.get())
        .err()
        .map(|e| e.0)
}

/// `physics::face_under_point` (0x442006): whether the point is inside face `number` (see
/// `physics/face_under_point.rs`): the carry.
// Converted: `rc2k_core::physics::ground_search::face_under_point`.
pub fn face_under_point_at(
    course: CourseReads<'_>,
    q: &GroundQuery<'_>,
    number: u32,
    switches: &crate::scene::draw_switches_store::DrawSwitches,
) -> Result<bool, Refused> {
    if let Some(why) = face_refusal(course, number) {
        return Err(BadInput(why).into());
    }
    q.number.set(number);
    // The face read once: its kind, flags, corners and edges.
    let f = crate::race::course_places::face_number(course, number)?;
    let trap = Trap::new();
    // One edge.
    let side = |from: u32, to: u32| -> Option<bool> {
        if trap.failed() {
            return None;
        }
        match edge_side(course, q, f.dword_at(from), f.dword_at(to)) {
            Ok(s) => (s.zero == Some(true)).then_some(s.carry),
            Err(e) => {
                trap.refuse(e);
                None
            }
        }
    };
    let test = ground_search::face_under_point(
        f.kind(),
        || f.switches(),
        q.settle_mark.get(),
        |o| f.dword_at(o),
        side,
    );
    trap.finish()?;
    let (half, corners) = match test {
        FaceTest::Kind | FaceTest::Flags(_) | FaceTest::Edge => return Ok(false),
        FaceTest::Inside { half, corners } => (half, corners),
    };
    q.slot.set(q.number.get());
    q.half.set(half);
    for (k, &o) in corners.iter().enumerate() {
        let corner = f.dword_at(o);
        q.corner_numbers[k].set(corner);
        q.face_corners[k].set(corner);
    }
    face_shares_at(course, q, switches)?;
    q.done.set(1);
    plane_height(q);
    Ok(true)
}

/// `search_record_faces`' refusal, worked out without writing.
pub fn record_refusal(course: CourseReads<'_>, record: u32) -> Option<&'static str> {
    let course_store = course;
    {
        if let Err(e) =
            ground_search::check_record(record, course_store.course.course_record_count.get())
        {
            return Some(e.0);
        }
        let Ok((first, last)) = record_faces(course_store, record) else {
            return Some("a course record outside every allocation");
        };
        ground_search::check_record_faces(first, last, course_store.course.course_face_count.get())
            .err()
            .map(|e| e.0)
    }
}

/// `physics::search_record_faces` (0x441FD9): faces +8 ..= +0xC of course record
/// `record`, each by `face_under_point`; the carry when one has the point.
// Converted: `rc2k_core::physics::ground_search::search_faces`.
pub fn search_record_faces_at(
    course: CourseReads<'_>,
    q: &GroundQuery<'_>,
    record: u32,
    switches: &crate::scene::draw_switches_store::DrawSwitches,
) -> Result<bool, Refused> {
    if let Some(why) = record_refusal(course, record) {
        return Err(BadInput(why).into());
    }
    let (first, last) = record_faces(course, record)?;
    let (found, _) = ground_search::search_faces(first, last, |number| {
        face_under_point_at(course, q, number, switches)
    })?;
    Ok(found)
}

/// `physics::search_cell` (0x441FBA): the cell of (`x`, `z`) (each >> 21; key
/// z << 11 + x), its record by `find_course_record`, then its faces. The carry when
/// found.
// Converted: `rc2k_core::physics::ground_search::cell_key`.
pub fn search_cell_at(
    course: CourseReads<'_>,
    q: &GroundQuery<'_>,
    x: u32,
    z: u32,
    switches: &crate::scene::draw_switches_store::DrawSwitches,
) -> Result<bool, Refused> {
    let (key, _) = ground_search::cell_key(x, z);
    let Some(record) = find_course_record(course, key)? else {
        return Ok(false);
    };
    search_record_faces_at(course, q, record, switches)
}

/// `physics::try_last_face` (0x441E0C): the saved last face (`last_face`); when it is a face (not -1,
/// below the count) `face_under_point` on it. The carry when found.
// Converted: `rc2k_core::physics::ground_search::last_face`.
pub fn try_last_face_at(
    course: CourseReads<'_>,
    q: &GroundQuery<'_>,
    switches: &crate::scene::draw_switches_store::DrawSwitches,
) -> Result<bool, Refused> {
    let last = q.last.get();
    if ground_search::last_face(last, course.course.course_face_count.get()).is_none() {
        return Ok(false);
    }
    face_under_point_at(course, q, last, switches)
}

/// `physics::ground_search` (0x441E2F), inside `pusha` / `popa`: `try_last_face`; else
/// `ground_slot` = -1 and `search_cell` on the point plus each of `CELLS` until one
/// finds it. `g_groundFound` 1 when found, else 0. Registers kept; the carry the found.
// Converted: `rc2k_core::physics::ground_search::ground_search`.
pub fn ground_search_at(
    course: CourseReads<'_>,
    q: &GroundQuery<'_>,
    switches: &crate::scene::draw_switches_store::DrawSwitches,
) -> Result<bool, Refused> {
    let found = ground_search::ground_search(
        || try_last_face_at(course, q, switches),
        || {
            q.slot.set(ground_search::NO_FACE);
        },
        || (q.x.get(), q.z.get()),
        |x, z| search_cell_at(course, q, x, z, switches),
    )?;
    q.found.set(u32::from(found));
    Ok(found)
}
