//! `scene::set_object_material` (0x45170E): what an object's drawing needs from its
//! kind's record, and for a kind that moves between two shapes, how far it has moved.
//!
//! `dword_623DD0` = 0 and the polygon's colour white; the object's kind
//! (`g_objectKinds[object]`) picks a record of 0x80 bytes at `unk_97DF30`, whose byte +0x20
//! goes to `dword_6214DC`; then the object's first face and number of faces (`dword_632B70`,
//! `dword_632F70`) to `g_objectFirstFace` and `g_objectFaceCount`. With that byte at 4 the
//! object has two shapes, the second right after the first: `dword_623DD0` = the bytes between
//! them (the next object's first point less this one's, times 12), and 0x45079A works out
//! the fraction `flt_623DD4` from a state of four dwords (+0x30 .. +0x3C of the record
//! `dword_5CCCD0` of 0x40 bytes at `dword_5CCCBC`: the clock when last here, the time into the
//! phase, the phase's length, the phase): the time since last here (the clock `dword_613830`,
//! at most 100) is added; while the time is not under the length, it loses the length, the
//! phase goes on by one of four, and the new length is a base (200, 400, 200, 400) plus a
//! random part of 400 (the drawing's random numbers, `next_drawing_random`, its top half times 400). The fraction: 0 in phase
//! 0, time over length in phase 1, 1 in phase 2, 1 less that in phase 3. The original has a
//! second way for when the four random parts are all 0, which they never are (it sets them
//! itself just before): left out. One caller, `scene::draw_object`. Refuses an object's number past
//! the tables and, for a moving kind, no state.
//!
//! Converted 2026-10-21: the Rust version is `set_object_material_plain`, through
//! `rc2k_core::scene::materials` (the fraction a plain float; the x87 version stays for the
//! Rust callers in unconverted code).

use crate::errors::BadInput;
use crate::math::next_random::next_drawing_random;
use crate::math::x87;
use crate::race::owner_shared::Shared;

// ---- game data ----
const OBJECTS: u32 = 0xFF;
pub const MATERIAL_LEN: usize = 0x80;
/// The state as the function works on it: the clock when last here (`dword_62DB54`),
/// then `dword_631B5C` the phase's length, `dword_631B60` the time into it,
/// `dword_631B64` the phase.
/// `dword_62DB30` ..: four bases, four random parts, and a 0.
const LENGTHS: [u32; 9] = [200, 400, 200, 400, 400, 400, 400, 400, 0];

// ---- Rust version ----

/// The face whose +0x30 .. +0x40 hold the state (`g_currentCourseFace`), or None without
/// the course faces.
fn state(course: &crate::race::owner_course::Course) -> Option<u32> {
    (course.course.course_faces.bits() != 0).then(|| course.course.current_course_face.bits())
}

/// The state's four dwords from +0x30 of the face.
const STATE: u32 = 0x30;

/// 0x45079A, on the state of face `face`.
fn step_shape(
    shared: &Shared,
    face: u32,
    scene: &crate::scene::owner::Scene,
) -> Result<(), BadInput> {
    use crate::race::course_places::{face_field, write_face_field};
    let b: [u8; 16] = face_field(face, STATE, "an object's state")?;
    let [last, mut time, mut length, mut phase]: [u32; 4] = std::array::from_fn(|k| {
        u32::from_le_bytes([b[4 * k], b[4 * k + 1], b[4 * k + 2], b[4 * k + 3]])
    });
    crate::session::write_bits(&scene.lights.work.phase_lengths, LENGTHS);
    let clock: u32 = shared.physics_tables.scene_clock.bits();
    time = time.wrapping_add(clock.wrapping_sub(last).min(100));
    while time >= length {
        time -= length;
        phase = (phase + 1) & 3;
        let part =
            ((next_drawing_random() as u64 * LENGTHS[4 + phase as usize] as u64) >> 32) as u32;
        length = part.wrapping_add(LENGTHS[phase as usize]);
    }
    let fraction = match phase {
        1 => x87::int_ratio(time as i32, length as i32),
        2 => 1f32.to_bits(),
        3 => x87::one_less_int_ratio(time as i32, length as i32),
        _ => 0f32.to_bits(),
    };
    shared.vars.move_fraction.set_bits(fraction);
    scene.lights.work.phase_clock.set_bits(clock);
    crate::session::write_first(&scene.objects.slots.phase_work, &[length, time, phase]);
    let mut b = [0u8; 16];
    for (k, v) in [clock, time, length, phase].into_iter().enumerate() {
        b[4 * k..4 * k + 4].copy_from_slice(&v.to_le_bytes());
    }
    write_face_field(face, STATE, b, "an object's state")?;
    Ok(())
}

pub fn set_object_material(ctx: &crate::race::view_scratch::DrawCtx<'_>) -> Result<(), BadInput> {
    let (race, scene) = (ctx.race, ctx.scene);
    let shared = &race.shared;
    let object: u32 = shared.switches.object_number.bits();
    if object >= OBJECTS {
        return Err(BadInput("an object's number past the tables"));
    }
    let kind: u32 = scene.objects.tables.object_kind(object);
    let material = crate::scene::polygon::materials_ptr()
        .offset(((kind as usize).wrapping_mul(MATERIAL_LEN)) as u32);
    let way = material.offset(0x20).read::<u8>("an object's material")? as u32;
    let state = state(&race.course);
    if way == 4 && state.is_none() {
        return Err(BadInput("a moving object without its state"));
    }
    shared.vars.move_by.set_bits(0u32);
    scene.drawing.scene.polygon_colour.set_bits(0x00FF_FFFFu32);
    shared.switches.other_way_kind.set_bits(way);
    if let (4, Some(state)) = (way, state) {
        let points: [u32; 2] = [
            scene.objects.slots.first_point[object as usize].get(),
            scene.objects.slots.first_point[object as usize + 1].get(),
        ];
        shared
            .vars
            .move_by
            .set_bits(points[1].wrapping_sub(points[0]).wrapping_mul(12));
        step_shape(shared, state, scene)?;
    }
    ctx.view
        .object
        .first_face
        .set_bits(scene.objects.slots.first_face[object as usize].get());
    ctx.view
        .object
        .face_count
        .set_bits(scene.objects.slots.face_counts[object as usize].get());
    Ok(())
}
