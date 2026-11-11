//! `scene::set_object_material` (0x45170E): what an object's drawing needs from its
//! kind's record, and for a kind that moves between two shapes, how far it has moved.
//!
//! `move_by` = 0 and the polygon's colour white; the object's kind picks a record of 0x80
//! bytes in `object_materials`, whose byte +0x20 goes to `other_way_kind`; then the object's
//! `first_face` and `face_counts` entries to the current object's first face and face count.
//! With that byte at 4 the object has two shapes, the second right after the first:
//! `move_by` = the bytes between them (the next object's first point less this one's, times
//! 12), and the fraction `move_fraction` is worked out from a state of four dwords (+0x30 ..
//! +0x3C of the current course face's record of 0x40 bytes: the clock when last here, the
//! time into the phase, the phase's length, the phase): the time since last here (the scene
//! clock, at most 100) is added; while the time is not under the length, it loses the length,
//! the phase goes on by one of four, and the new length is a base (200, 400, 200, 400) plus a
//! random part of 400 (the drawing's random numbers, `next_drawing_random`, its top half
//! times 400). The fraction: 0 in phase 0, time over length in phase 1, 1 in phase 2, 1 less
//! that in phase 3. The original has a second way for when the four random parts are all 0,
//! which they never are (it sets them itself just before): left out. One caller,
//! `scene::draw_object`. Refuses an object's number past the tables and, for a moving kind,
//! no state.
//!
//! The work is `set_object_material_plain`, through `rc2k_core::scene::materials` (the
//! fraction a plain float; the x87 version stays for the callers in unconverted code).

use crate::errors::BadInput;
use crate::math::next_random::next_drawing_random;
use crate::math::x87;
use crate::math::x87_float as bh;

// ---- game data ----
const OBJECTS: u32 = 0xFF;
pub const MATERIAL_LEN: usize = 0x80;
/// The state as the function works on it: the clock when last here (`phase_clock`), then
/// the `phase_work` cells: the phase's length, the time into it, the phase. The
/// `phase_lengths` table holds four bases, four random parts, and a 0.
const LENGTHS: [u32; 9] = [200, 400, 200, 400, 400, 400, 400, 400, 0];

// ---- Rust version ----

/// The face whose +0x30 .. +0x40 hold the state (the view's current course face), or
/// None without the course faces. (The original read the face the previous view drew last
/// when an object of two shapes came before the view's first face; no kind in the game's
/// files has that material, so it never comes into play.)
fn state(
    course: &crate::race::owner_course::Course,
    face: &crate::race::face_work::FaceWork,
) -> Option<u32> {
    (course.course.course_faces.get() != 0).then(|| face.course_face.get())
}

/// The state's four dwords from +0x30 of the face.
const STATE: u32 = 0x30;

/// The fraction of the way between the two shapes, on the state of face `face`.
/// The fraction into `fraction_out` (the view's `ObjectPlace::move_fraction`).
fn step_shape(
    course: &crate::race::owner_course::Course,
    clock: u32,
    face: u32,
    scene: &crate::scene::owner::Scene,
    fraction_out: &crate::session::Dword,
) -> Result<(), BadInput> {
    use crate::race::course_places::{face_field, write_face_field};
    let read = || -> Result<[u32; 4], BadInput> {
        let b: [u8; 16] = face_field(course, face, STATE)?;
        Ok(std::array::from_fn(|k| {
            u32::from_le_bytes([b[4 * k], b[4 * k + 1], b[4 * k + 2], b[4 * k + 3]])
        }))
    };
    let [mut last, mut time, mut length, mut phase] = read()?;
    crate::session::write_bits(&scene.lights.work.phase_lengths, LENGTHS);
    if time.wrapping_add(clock.wrapping_sub(last).min(100)) >= length {
        // The phase advances: in the list's order, after the views before this one have
        // stepped the face (the mirror then finds the view's new state and does not advance
        // it again, as when one thread draws them in turn).
        crate::workers::wait_for_earlier_runs();
        [last, time, length, phase] = read()?;
    }
    time = time.wrapping_add(clock.wrapping_sub(last).min(100));
    while time >= length {
        time -= length;
        phase = (phase + 1) & 3;
        let part =
            ((next_drawing_random() as u64 * LENGTHS[4 + phase as usize] as u64) >> 32) as u32;
        length = part.wrapping_add(LENGTHS[phase as usize]);
    }
    let fraction = match phase {
        1 => x87::int_ratio(time as i32, length as i32).to_bits(),
        2 => 1f32.to_bits(),
        3 => bh::st(bh::ONE - bh::int(time as i32) / bh::int(length as i32)),
        _ => 0f32.to_bits(),
    };
    fraction_out.set(fraction);
    crate::session::write_first(&scene.objects.slots.phase_work, &[length, time, phase]);
    let mut b = [0u8; 16];
    for (k, v) in [clock, time, length, phase].into_iter().enumerate() {
        b[4 * k..4 * k + 4].copy_from_slice(&v.to_le_bytes());
    }
    write_face_field(course, face, STATE, b)?;
    Ok(())
}

pub fn set_object_material(ctx: &crate::race::view_scratch::DrawCtx<'_>) -> Result<(), BadInput> {
    let (race, scene) = (ctx.race, ctx.scene);
    let object: u32 = ctx.view.cars.object_number.get();
    if object >= OBJECTS {
        return Err(BadInput("an object's number past the tables"));
    }
    let kind: u32 = scene.objects.tables.object_kind(object);
    let material = crate::scene::polygon::materials_ptr()
        .offset(((kind as usize).wrapping_mul(MATERIAL_LEN)) as u32);
    let way = material.offset(0x20).read::<u8>("an object's material")? as u32;
    let state = state(&race.course, &ctx.view.face);
    if way == 4 && state.is_none() {
        return Err(BadInput("a moving object without its state"));
    }
    ctx.view.object.move_by.set(0);
    ctx.view.polygon.polygon_colour.set(0x00FF_FFFFu32);
    ctx.view.cars.other_way_kind.set(way);
    if let (4, Some(state)) = (way, state) {
        let points: [u32; 2] = [
            scene.objects.slots.first_point[object as usize].get(),
            scene.objects.slots.first_point[object as usize + 1].get(),
        ];
        ctx.view
            .object
            .move_by
            .set(points[1].wrapping_sub(points[0]).wrapping_mul(12));
        step_shape(
            &race.course,
            ctx.clock,
            state,
            scene,
            &ctx.view.object.move_fraction,
        )?;
    }
    ctx.view
        .object
        .first_face
        .set(scene.objects.slots.first_face[object as usize].get());
    ctx.view
        .object
        .face_count
        .set(scene.objects.slots.face_counts[object as usize].get());
    Ok(())
}
