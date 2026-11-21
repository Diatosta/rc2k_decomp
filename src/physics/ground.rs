//! The ground's search: which course face lies under the point (`ground_x`, `ground_z`,
//! whole-number x and z), and the shares and height there. `physics::ground_search` tries the
//! last face found (`try_last_face_at`), then the nine grid cells around the point
//! (`physics::search_cell`: the cell's course record, `physics::find_course_record`, then its
//! faces, `search_record_faces_at`), each face by `physics::face_under_point` (its edges
//! by `physics::edge_side`; when inside, `physics::face_shares` and `physics::plane_height`).
//!
//! The cores take what they read (the face, the record, the point) and return whether the point
//! was found.

use crate::errors::{BadInput, Refused};
use crate::physics::cells::Trap;
use crate::physics::edge_side::edge_side;
use crate::physics::find_course_record::find_course_record;
use crate::physics::ground_query::{GroundEnv, GroundSearch};
use crate::physics::plane_height::plane_height;
use crate::race::owner_course::CourseReads;
use rc2k_core::physics::ground_search::{self, FaceTest};
use std::cell::RefCell;

/// The first and last faces (the record's second and third dwords) of course record `record`.
fn record_faces(course: CourseReads<'_>, record: u32) -> Result<(u32, u32), BadInput> {
    let face = |offset| crate::race::course_places::record_field::<u32>(course, record, offset);
    Ok((face(8)?, face(0xC)?))
}

/// `physics::face_shares`: the three corners named by `corner_numbers`, the differences from the
/// first one, and the shares by the x87 (`math::x87::face_shares`).
pub fn face_shares_at(env: GroundEnv<'_>, q: &mut GroundSearch) -> Result<(), Refused> {
    let mut corner = [[0u32; 3]; 3];
    for (k, c) in corner.iter_mut().enumerate() {
        *c = crate::race::scene_points::point_number(env.course, env.points, q.corner_numbers[k])?;
        q.corners[3 * k..3 * k + 3].copy_from_slice(c);
    }
    let (x, z) = (q.x, q.z);
    let x_differences = [
        x.wrapping_sub(corner[0][0]),
        corner[1][0].wrapping_sub(corner[0][0]),
        corner[2][0].wrapping_sub(corner[0][0]),
    ];
    let z_differences = [
        z.wrapping_sub(corner[0][2]),
        corner[1][2].wrapping_sub(corner[0][2]),
        corner[2][2].wrapping_sub(corner[0][2]),
    ];
    let shares = crate::math::x87::face_shares(&crate::math::x87::FaceDifferences {
        x: x_differences.map(|d| d as i32),
        z: z_differences.map(|d| d as i32),
        scale: &env.course.numbers.k65536,
    });
    q.plane_f = shares.plane_f;
    q.share_s = shares.share_s as u32;
    q.plane_g = shares.plane_g;
    q.share_t = shares.share_t as u32;
    Ok(())
}

/// `face_under_point`'s refusal, worked out without writing.
fn face_refusal(course: CourseReads<'_>, face: u32) -> Option<&'static str> {
    ground_search::check_face(face, course.course.get().course_face_count)
        .err()
        .map(|e| e.0)
}

/// `physics::face_under_point`: whether the point is inside face `number` (see
/// `physics/face_under_point.rs`).
// Core: `rc2k_core::physics::ground_search::face_under_point`.
fn face_under_point_at(
    env: GroundEnv<'_>,
    q: &mut GroundSearch,
    number: u32,
) -> Result<bool, Refused> {
    let course = env.course;
    if let Some(why) = face_refusal(course, number) {
        return Err(BadInput(why).into());
    }
    // The face read once: its kind, flags, corners and edges.
    let f = crate::race::course_places::face_number(course, number)?;
    let trap = Trap::new();
    let point = (q.x, q.z);
    // One edge.
    let side = |from: u32, to: u32| -> Option<bool> {
        if trap.failed() {
            return None;
        }
        match edge_side(env, point, f.dword_at(from), f.dword_at(to)) {
            Ok(side) => side,
            Err(e) => {
                trap.refuse(e);
                None
            }
        }
    };
    let test = ground_search::face_under_point(
        f.kind(),
        || f.switches(),
        env.mark,
        |o| f.dword_at(o),
        side,
    );
    trap.finish()?;
    let (half, corners) = match test {
        FaceTest::Kind | FaceTest::Flags(_) | FaceTest::Edge => return Ok(false),
        FaceTest::Inside { half, corners } => (half, corners),
    };
    q.slot = number;
    q.half = half;
    for (k, &o) in corners.iter().enumerate() {
        q.corner_numbers[k] = f.dword_at(o);
    }
    face_shares_at(env, q)?;
    plane_height(q);
    Ok(true)
}

/// `search_record_faces`' refusal, worked out without writing.
fn record_refusal(course: CourseReads<'_>, record: u32) -> Option<&'static str> {
    let course_store = course;
    {
        if let Err(e) =
            ground_search::check_record(record, course_store.course.get().course_record_count)
        {
            return Some(e.0);
        }
        let Ok((first, last)) = record_faces(course_store, record) else {
            return Some("a course record outside every allocation");
        };
        ground_search::check_record_faces(first, last, course_store.course.get().course_face_count)
            .err()
            .map(|e| e.0)
    }
}

/// `physics::search_record_faces`: the first to the last face of course
/// record `record`, each by `face_under_point`; true when one has the point.
// Core: `rc2k_core::physics::ground_search::search_faces`.
pub(super) fn search_record_faces_at(
    env: GroundEnv<'_>,
    q: &mut GroundSearch,
    record: u32,
) -> Result<bool, Refused> {
    let course = env.course;
    if let Some(why) = record_refusal(course, record) {
        return Err(BadInput(why).into());
    }
    let (first, last) = record_faces(course, record)?;
    let (found, _) =
        ground_search::search_faces(first, last, |number| face_under_point_at(env, q, number))?;
    Ok(found)
}

/// `physics::search_cell`: the cell of (`x`, `z`) (each >> 21; key
/// z << 11 + x), its record by `find_course_record`, then its faces. True when found.
// Core: `rc2k_core::physics::ground_search::cell_key`.
fn search_cell_at(
    env: GroundEnv<'_>,
    q: &mut GroundSearch,
    x: u32,
    z: u32,
) -> Result<bool, Refused> {
    let (key, _) = ground_search::cell_key(x, z);
    let Some(record) = find_course_record(env.course, key)? else {
        return Ok(false);
    };
    search_record_faces_at(env, q, record)
}

/// `physics::try_last_face`: the saved last face (`last_face`); when it is a face (not -1,
/// below the count) `face_under_point` on it. True when found.
// Core: `rc2k_core::physics::ground_search::last_face`.
pub(super) fn try_last_face_at(env: GroundEnv<'_>, q: &mut GroundSearch) -> Result<bool, Refused> {
    let last = q.last;
    if ground_search::last_face(last, env.course.course.get().course_face_count).is_none() {
        return Ok(false);
    }
    face_under_point_at(env, q, last)
}

/// `physics::ground_search`: `try_last_face_at`; else `ground_slot` = -1 and `search_cell` on the
/// point plus each of `CELLS` until one finds it. `ground_found` is 1 when found, else 0; the
/// result says whether it was found.
// Core: `rc2k_core::physics::ground_search::ground_search`.
pub fn ground_search_at(env: GroundEnv<'_>, q: &mut GroundSearch) -> Result<bool, Refused> {
    // The steps take turns on the record; none runs while another does.
    let q = RefCell::new(q);
    let found = ground_search::ground_search(
        || try_last_face_at(env, &mut q.borrow_mut()),
        || {
            q.borrow_mut().slot = ground_search::NO_FACE;
        },
        || {
            let q = q.borrow();
            (q.x, q.z)
        },
        |x, z| search_cell_at(env, &mut q.borrow_mut(), x, z),
    )?;
    q.borrow_mut().found = u32::from(found);
    crate::debug::timing::note_ground_search(found);
    Ok(found)
}
