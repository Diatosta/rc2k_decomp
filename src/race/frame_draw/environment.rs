//! The EAX sound environment: the weight of the course faces round the first car.

use super::*;
use crate::errors::BadInput;
use crate::race::owner::Race;
use crate::race::owner_course::CourseReads;

/// The game's table root of a float (`sqrt` by `g_rootTable` x `g_exponentRootTable`),
/// times 1/65536 (`race().course.numbers.one_over_65536`), as stored.
pub(super) fn table_root(course: CourseReads<'_>, v: f32) -> f32 {
    let v = v.to_bits();
    // g_rootTable: 256 floats by mantissa bits; g_exponentRootTable: by exponent
    let root = &course.lists.root_table[((v >> 15) & 0xFF) as usize];
    let scale = &course.lists.exponent_root_table[((v & 0x7F80_0000) >> 23) as usize];
    crate::math::x87::mul2(root, scale, &course.numbers.one_over_65536)
}

/// A point's x and z less car 0's (`g_carBlockStore` +0x134, +0x13C), and its y. (The
/// original also kept x and z in `CONTACT_WORK.eax_point_x` / `eax_point_z`, which nothing
/// read: locals here.)
pub(super) fn point_from_car(
    course: CourseReads<'_>,
    points: &crate::race::scene_points::OwnPoints,
    point: u32,
) -> Result<(u32, u32, u32), BadInput> {
    let [px, py, pz] = crate::race::scene_points::point_number(course, points, point)?;
    let x = px.wrapping_sub(crate::race::frame_view::car(0)?.position[0].get());
    let z = pz.wrapping_sub(crate::race::frame_view::car(0)?.position[2].get());
    Ok((x, py, z))
}

/// The weight's end: `v` held to 16.0 (`race().course.numbers.sixteen`; kept when below or
/// not a number), added to `sum` (the original's `g_eaxEnvironment`; it also counted the
/// faces in `g_eaxFaceCount`, which nothing read).
pub(super) fn add_eax_weight(course: CourseReads<'_>, sum: &mut u32, v: f32) {
    let sixteen = course.numbers.sixteen.get();
    let w = if crate::math::x87::below_or_unordered(v, sixteen) {
        v
    } else {
        sixteen
    };
    *sum = crate::math::x87::add(w, *sum).to_bits();
}

/// `race::eax_face_weight` (0x45C881): the weight of `face` in the EAX sound environment
/// (`g_eaxEnvironment`), by its distance from car 0 (x and z): kind 0xFF with an object (+1) below
/// 0x38: its +0x18 byte over the distance to its first point; kinds below 0x10 with |+0x30| below
/// 0.25: 3.0 over the distance to the middle of its second and third points; each held to 16 (see
/// `add_eax_weight`). Distances by the game's root tables. (The original kept the middle and the
/// distance in `CONTACT_WORK.eax_mid_x` / `eax_mid_z` / `eax_distance`, which nothing read: locals
/// here.)
pub fn eax_face_weight(
    course: CourseReads<'_>,
    points: &crate::race::scene_points::OwnPoints,
    sum: &mut u32,
    face: u32,
) -> Result<(), BadInput> {
    use crate::math::x87::{
        abs_below_or_unordered, float_square_sum, int_plus_times, int_square_sum,
    };
    use crate::math::x87::{div, from_int, int_over};
    {
        // The face read once for its kind, objects, switches and first point.
        let read = crate::race::course_places::face_number(course, face)?;
        let kind = read.kind();
        if kind == 0xFF {
            if face_field::<u8>(course, face, 1)? >= 0x38 {
                return Ok(());
            }
            let (x, _, z) = point_from_car(course, points, face_field(course, face, 4)?)?;
            let distance = table_root(course, int_square_sum(x as i32, z as i32));
            let strength = face_field::<u8>(course, face, 0x18)? as u32;
            add_eax_weight(course, sum, int_over(strength as i32, distance));
            return Ok(());
        }
        if kind >= 0x10 {
            return Ok(());
        }
        if !abs_below_or_unordered(
            face_field::<u32>(course, face, 0x30)?,
            course.numbers.quarter.get(),
        ) {
            return Ok(());
        }
        let (x1, _, z1) = point_from_car(course, points, face_field(course, face, 8)?)?;
        let (mx, mz) = (from_int(x1 as i32), from_int(z1 as i32));
        let (x2, _, z2) = point_from_car(course, points, face_field(course, face, 0xC)?)?;
        let half = course.numbers.half.get();
        let mx = int_plus_times(x2 as i32, mx, half);
        let mz = int_plus_times(z2 as i32, mz, half);
        let distance = table_root(course, float_square_sum(mx, mz));
        add_eax_weight(course, sum, div(&course.numbers.three, distance));
    }
    Ok(())
}

/// `race::eax_environment` (0x45C7EE): the view's EAX weight, returned (the view's output:
/// `race::draw_view_scene`): from 0.0, the 3 x 3 grid cells around the middle (from
/// `hold_to_limit(race().course.stage_prefs.grid_reach)` - 1 on each axis; 0x45C84C each): every
/// face of each cell's course record (numbers below 0x0FFFFFFF) weighed by `eax_face_weight`. The
/// original summed straight into `g_eaxEnvironment`, which the audio thread reads at its own pace
/// (`audio::update_voices`): it could take a sum half made (`notes/bugs.md`).
pub fn eax_environment(
    race: &Race,
    course: CourseReads<'_>,
    points: &crate::race::scene_points::OwnPoints,
    view: &crate::race::view_grid::ViewGrid,
) -> Result<u32, BadInput> {
    let mut sum = 0u32;
    {
        // Converted: `rc2k_core::race::view::{cells_from, record_faces}`.
        use rc2k_core::race::view::{NO_RECORD, cells_from, record_faces};
        for cell in cells_from(
            hold_to_limit(&race.shared, course.stage_prefs.grid_reach.get()).wrapping_sub(1),
            3,
        ) {
            // 0x45C84C.
            let record = view.cell(cell)?.get();
            if record >= NO_RECORD {
                continue;
            }
            let mut face = record_field(course, record, 8)?;
            if record_faces(face, record_field(course, record, 0xC)?).is_none() {
                continue;
            }
            loop {
                eax_face_weight(course, points, &mut sum, face)?;
                face = face.wrapping_add(1);
                if face > record_field(course, record, 0xC)? {
                    break;
                }
            }
        }
    }
    Ok(sum)
}
