//! The ground's search: which course face lies under the point (`dword_639824`,
//! `dword_63982C`, whole-number x and z), and the shares and height there (IDA,
//! 2026-10-17). `physics::ground_search` tries the last face found (`physics::try_last_face`), then
//! the nine grid cells around the point (`physics::search_cell`: the cell's course record,
//! `physics::find_course_record`, then its faces, `physics::search_record_faces`), each
//! face by `physics::face_under_point` (its edges by `physics::edge_side`; when inside,
//! `physics::face_shares` and `physics::plane_height`).
//!
//! Plain since 2026-10-23 (phase 3): the cores take what they read (the face, the record,
//! the point) and return the carry; the registers the originals leave are read by none of
//! their callers (`ground_search` keeps them all: `pusha` / `popa`).

use crate::errors::{BadInput, Refused};
use crate::physics::cells::Trap;
use crate::physics::edge_side::edge_side;
use crate::physics::find_course_record::find_course_record;
use crate::physics::plane_height::plane_height;
use crate::race::owner::Race;
use crate::race::owner_course::Course;
use rc2k_core::physics::ground_search::{self, FaceTest};

/// The first and last faces (+8, +0xC) of course record `record`.
fn record_faces(record: u32) -> Result<(u32, u32), BadInput> {
    let face =
        |offset| crate::race::course_places::record_field::<u32>(record, offset, "a course record");
    Ok((face(8)?, face(0xC)?))
}

/// `physics::face_shares` (0x442323): the three corners named by `dword_639890` ..
/// `98` copied (0x639708 ..), the point (0x6396FC, 0x639704), the differences, and the
/// shares by the x87 (`math::x87_run::face_shares`; the `rcpss` way with the SSE switch).
/// The registers it leaves (eax the last difference, ebx, ecx the third corner's y, z)
/// are read by none of its callers.
pub fn face_shares_at(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let race_owner = race;
    let mut corner = [[0u32; 3]; 3];
    for (k, c) in corner.iter_mut().enumerate() {
        *c = crate::race::scene_points::point_number(
            race_owner.physics.ground.face_corner_numbers[k].get(),
        )?;
        for (n, &v) in c.iter().enumerate() {
            race_owner.physics.ground.face_corners[3 * k + n].set(v);
        }
    }
    let (x, z) = (
        race_owner.physics.ground.ground_x.get(),
        race_owner.physics.ground.ground_z.get(),
    );
    race_owner.physics.ground.face_point_x.set(x);
    race_owner.physics.ground.face_point_z.set(z);
    let d = [
        (
            &race_owner.physics.ground.corner_x_differences[0],
            x.wrapping_sub(corner[0][0]),
        ),
        (
            &race_owner.physics.ground.corner_x_differences[1],
            corner[1][0].wrapping_sub(corner[0][0]),
        ),
        (
            &race_owner.physics.ground.corner_x_differences[2],
            corner[2][0].wrapping_sub(corner[0][0]),
        ),
        (
            &race_owner.physics.ground.corner_z_differences[0],
            z.wrapping_sub(corner[0][2]),
        ),
        (
            &race_owner.physics.ground.corner_z_differences[1],
            corner[1][2].wrapping_sub(corner[0][2]),
        ),
        (
            &race_owner.physics.ground.corner_z_differences[2],
            corner[2][2].wrapping_sub(corner[0][2]),
        ),
        (
            &race_owner.physics.ground.corner_y_differences[0],
            corner[1][1].wrapping_sub(corner[0][1]),
        ),
        (
            &race_owner.physics.ground.corner_y_differences[1],
            corner[2][1].wrapping_sub(corner[0][1]),
        ),
    ];
    for &(cell, value) in &d {
        cell.set(value);
    }
    let sse = scene.draw_switches.use_sse.get() != 0;
    let i = [
        d[0].1,
        d[1].1,
        d[2].1,
        d[3].1,
        d[4].1,
        d[5].1,
        race_owner.course.numbers.k65536.bits(),
    ];
    let mut o = [0u32; 8];
    crate::math::x87_run::face_shares(&i, &mut o, sse);
    race_owner.physics.ground.face_share_a.set_bits(o[0]);
    race_owner.physics.ground.face_share_b.set_bits(o[1]);
    race_owner.physics.ground.face_share_c.set_bits(o[2]);
    race_owner.physics.ground.plane_f.set_bits(o[3]);
    race_owner.physics.ground.share_s.set(o[4]);
    race_owner.physics.ground.plane_g.set_bits(o[5]);
    race_owner.physics.ground.share_t.set(o[6]);
    if sse {
        race_owner.physics.ground.face_share_scale.set_bits(o[7]);
    }
    Ok(())
}

/// `face_under_point`'s refusal, worked out without writing.
pub fn face_refusal(course: &Course, face: u32) -> Option<&'static str> {
    ground_search::check_face(face, course.course.course_face_count.get())
        .err()
        .map(|e| e.0)
}

/// `physics::face_under_point` (0x442006): whether the point is inside face `number`
/// (ebx; see `physics/face_under_point.rs`): the carry. The registers it leaves are read
/// by none of its callers (`search_record_faces`, `try_last_face`).
// Converted: `rc2k_core::physics::ground_search::face_under_point`.
pub fn face_under_point_at(
    race: &Race,
    number: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<bool, Refused> {
    let physics = &race.physics;
    let race_owner = race;
    if let Some(why) = face_refusal(&race_owner.course, number) {
        return Err(BadInput(why).into());
    }
    race_owner.course.course.ground_number.set(number);
    // The face read once: its kind, flags, corners and edges.
    let f = crate::race::course_places::face_number(number, "a course face")?;
    let trap = Trap::new();
    // One edge.
    let side = |from: u32, to: u32| -> Option<bool> {
        if trap.failed() {
            return None;
        }
        match edge_side(&race_owner.physics, f.dword_at(from), f.dword_at(to)) {
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
        race_owner.physics.ground_probe.settle_mark.get(),
        |o| f.dword_at(o),
        side,
    );
    trap.finish()?;
    let (half, corners) = match test {
        FaceTest::Kind | FaceTest::Flags(_) | FaceTest::Edge => return Ok(false),
        FaceTest::Inside { half, corners } => (half, corners),
    };
    race_owner
        .course
        .course
        .ground_slot
        .set(race_owner.course.course.ground_number.get());
    race_owner.course.course.ground_half.set(half);
    for (k, &o) in corners.iter().enumerate() {
        let corner = f.dword_at(o);
        race_owner.physics.ground.face_corner_numbers[k].set(corner);
        race_owner.course.course.ground_corners[k].set(corner);
    }
    face_shares_at(race, scene)?;
    race_owner.physics.ground.face_under_point_done.set(1);
    plane_height(physics);
    Ok(true)
}

/// `search_record_faces`' refusal, worked out without writing.
pub fn record_refusal(course: &Course, record: u32) -> Option<&'static str> {
    let course_store = course;
    {
        if let Err(e) =
            ground_search::check_record(record, course_store.course.course_record_count.get())
        {
            return Some(e.0);
        }
        let Ok((first, last)) = record_faces(record) else {
            return Some("a course record outside every allocation");
        };
        ground_search::check_record_faces(first, last, course_store.course.course_face_count.get())
            .err()
            .map(|e| e.0)
    }
}

/// `physics::search_record_faces` (0x441FD9): faces +8 ..= +0xC of course record
/// `record` (ebx), each by `face_under_point`; the carry when one has the point.
// Converted: `rc2k_core::physics::ground_search::search_faces`.
pub fn search_record_faces_at(
    race: &Race,
    record: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<bool, Refused> {
    if let Some(why) = record_refusal(&race.course, record) {
        return Err(BadInput(why).into());
    }
    let (first, last) = record_faces(record)?;
    let (found, _) = ground_search::search_faces(first, last, |number| {
        face_under_point_at(race, number, scene)
    })?;
    Ok(found)
}

/// `physics::search_cell` (0x441FBA): the cell of (`x`, `z`) (eax, ecx; each >> 21; key
/// z << 11 + x), its record by `find_course_record`, then its faces. The carry when
/// found.
// Converted: `rc2k_core::physics::ground_search::cell_key`.
pub fn search_cell_at(
    race: &Race,
    x: u32,
    z: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<bool, Refused> {
    let (key, _) = ground_search::cell_key(x, z);
    let Some(record) = find_course_record(&race.course, key)? else {
        return Ok(false);
    };
    search_record_faces_at(race, record, scene)
}

/// `physics::try_last_face` (0x441E0C): ebx = `dword_5CCC78`; when it is a face (not -1,
/// below the count) `face_under_point` on it. The carry when found.
// Converted: `rc2k_core::physics::ground_search::last_face`.
pub fn try_last_face_at(race: &Race, scene: &crate::scene::owner::Scene) -> Result<bool, Refused> {
    let last = race.course.course.ground_face_last.get();
    if ground_search::last_face(last, race.course.course.course_face_count.get()).is_none() {
        return Ok(false);
    }
    face_under_point_at(race, last, scene)
}

/// `physics::ground_search` (0x441E2F), inside `pusha` / `popa`: `try_last_face`; else
/// `dword_5CCC74` = -1 and `search_cell` on the point plus each of `CELLS` until one
/// finds it. `g_groundFound` 1 when found, else 0. Registers kept; the carry the found.
// Converted: `rc2k_core::physics::ground_search::ground_search`.
pub fn ground_search_at(race: &Race, scene: &crate::scene::owner::Scene) -> Result<bool, Refused> {
    let found = ground_search::ground_search(
        || try_last_face_at(race, scene),
        || {
            race.course.course.ground_slot.set(ground_search::NO_FACE);
        },
        || {
            (
                race.physics.ground.ground_x.get(),
                race.physics.ground.ground_z.get(),
            )
        },
        |x, z| search_cell_at(race, x, z, scene),
    )?;
    race.physics.ground.ground_found.set(u32::from(found));
    Ok(found)
}
