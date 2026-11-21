//! `scene::set_object_material`: what an object's drawing needs from its kind's record, and for a
//! kind that moves between two shapes, how far it has moved.
//!
//! `move_by` = 0 and the polygon's colour white; the object's kind picks a record of 0x80 bytes in
//! `object_materials`, whose other-way byte goes to `other_way_kind`; then the object's
//! `first_face` and `face_counts` entries to the current object's first face and face count. With
//! that byte at 4 the object has two shapes, the second right after the first: `move_by` = the
//! bytes between them (the next object's first point less this one's, times 12), and the fraction
//! `move_fraction` is worked out from a state of four dwords (`STATE` on, of the current course
//! face's record of 0x40 bytes: the clock when last here, the time into the phase, the phase's
//! length, the phase): the time since last here (the scene clock, at most 100) is added; while the
//! time is not under the length, it loses the length, the phase goes on by one of four, and the new
//! length is a base (200, 400, 200, 400) plus a random part of 400 (the drawing's random numbers,
//! `next_drawing_random`, its top half times 400). The fraction: 0 in phase 0, time over length in
//! phase 1, 1 in phase 2, 1 less that in phase 3. There is no second way for when the four random
//! parts are all 0: they never are. Refuses an object's number past the tables and, for a moving
//! kind, no state.
//!
//! The work is `set_object_material_plain`, through `rc2k_core::scene::materials` (the fraction a
//! plain float; the x87 version stays for the callers in unconverted code).

use crate::errors::BadInput;
use crate::math::x87;
use crate::math::x87_float as bh;

// ---- game data ----
const OBJECTS: u32 = 0xFF;
/// The phases' lengths: four bases, four random parts, and a 0. Neither
/// `LightWork::phase_lengths` nor `ObjectTables::phase_work` is stored to: nothing reads them.
const LENGTHS: [u32; 9] = [200, 400, 200, 400, 400, 400, 400, 400, 0];

/// The face whose record holds the state (the view's current course face), or None
/// without the course faces. (An object of two shapes before the view's first face has no face
/// to read; no kind in the game's files has that material, so it never comes into play.)
fn state(
    data: &crate::race::course_places_store::CourseData,
    face: &crate::race::face_work::FaceWork,
) -> Option<u32> {
    crate::race::course_places::held(data).then(|| face.borrow().course_face)
}

/// Where the state's four dwords start in the face's record, in bytes.
const STATE: u32 = 0x30;

/// The fraction of the way between the two shapes, on the state of face `face`.
/// The fraction into the view's `ObjectPlace::move_fraction`.
fn step_shape(
    course: crate::race::owner_course::CourseReads<'_>,
    random: &crate::math::next_random::Randoms,
    clock: u32,
    face: u32,
    object: &crate::scene::object_place::ObjectPlace,
) -> Result<(), BadInput> {
    use crate::race::course_places::{face_field, write_face_field};
    let read = || -> Result<[u32; 4], BadInput> {
        let b: [u8; 16] = face_field(course, face, STATE)?;
        Ok(std::array::from_fn(|k| {
            u32::from_le_bytes([b[4 * k], b[4 * k + 1], b[4 * k + 2], b[4 * k + 3]])
        }))
    };
    let [mut last, mut time, mut length, mut phase] = read()?;
    if time.wrapping_add(clock.wrapping_sub(last).min(100)) >= length {
        // The phase advances: in the list's order, after the views before this one have
        // stepped the face (the mirror then finds the view's new state and does not advance
        // it again, as when one thread draws them in turn).
        crate::memory::draw_order::wait_for_earlier_runs();
        [last, time, length, phase] = read()?;
    }
    time = time.wrapping_add(clock.wrapping_sub(last).min(100));
    while time >= length {
        time -= length;
        phase = (phase + 1) & 3;
        let part =
            ((random.next_drawing() as u64 * LENGTHS[4 + phase as usize] as u64) >> 32) as u32;
        length = part.wrapping_add(LENGTHS[phase as usize]);
    }
    let fraction = match phase {
        1 => x87::int_ratio(time as i32, length as i32),
        2 => 1.0,
        3 => bh::stf(bh::ONE - bh::int(time as i32) / bh::int(length as i32)),
        _ => 0.0,
    };
    object.borrow_mut().move_fraction = fraction.to_bits();
    let mut b = [0u8; 16];
    for (k, v) in [clock, time, length, phase].into_iter().enumerate() {
        b[4 * k..4 * k + 4].copy_from_slice(&v.to_le_bytes());
    }
    write_face_field(course, face, STATE, b)?;
    Ok(())
}

pub fn set_object_material(ctx: &crate::race::view_scratch::DrawCtx<'_>) -> Result<(), BadInput> {
    let object: u32 = ctx.view.cars.borrow().object_number;
    if object >= OBJECTS {
        return Err(BadInput("an object's number past the tables"));
    }
    let kind: u32 = ctx.reads.object_tables.object_kind(object);
    let way = u32::from(crate::scene::polygon::material_run_byte(
        &ctx.reads.buffers.polygon_tables,
        kind,
        0x20,
    )?);
    let state = state(&ctx.reads.stage.course, &ctx.view.face);
    if way == 4 && state.is_none() {
        return Err(BadInput("a moving object without its state"));
    }
    ctx.view.object.borrow_mut().move_by = 0;
    ctx.view.polygon.scalars.borrow_mut().polygon_colour = 0x00FF_FFFFu32;
    ctx.view.cars.borrow_mut().other_way_kind = way;
    if let (4, Some(state)) = (way, state) {
        let points: [u32; 2] = [
            ctx.reads.object_slots.first_point(object as usize),
            ctx.reads.object_slots.first_point(object as usize + 1),
        ];
        ctx.view.object.borrow_mut().move_by = points[1].wrapping_sub(points[0]).wrapping_mul(12);
        step_shape(
            ctx.reads.course(),
            &ctx.race.shared.random,
            ctx.clock,
            state,
            &ctx.view.object,
        )?;
    }
    ctx.view.object.borrow_mut().first_face = ctx.reads.object_slots.first_face(object as usize);
    ctx.view.object.borrow_mut().face_count = ctx.reads.object_slots.face_count(object as usize);
    Ok(())
}
