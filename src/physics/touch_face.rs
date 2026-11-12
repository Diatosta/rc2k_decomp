//! `physics::touch_face` (0x45E9E0): the course face touched (`face_number`, in
//! `g_courseFaces`, 0x40 bytes each) marked and listed. A face whose byte +0x11 is not 0,
//! or a full list (`g_litFaceCount` at 0x100), is left alone. Else its number goes into
//! `g_litFaces[count]` and by the contact kind `contact_kind`:
//! - 9 and 1, unless `speed` is below `eight` (or unordered; then nothing more),
//!   and 2, 3, 8: bytes +9 = 0, +0x11 = 1, +0x13 = 0, +0x12 = `face_angle`'s low byte;
//! - 4: +0x11 = `face_speed` >> 14 held to 0x40, +0x13 = 0, +0x12 as above;
//! - 0xFF: one in about 85 (`math::next_random` & 0xFF below 3) +0x11 = 0xF, counted, no
//!   sound; else nothing more;
//! - the rest (10, 5, 6, 7, ...): nothing to the face.
//!
//! Then `physics::place_sound_object` and the count + 1.
//!
//! Callers: `physics::face_contact` (six). Refuses (before anything) a count past 0x100 (the list's
//! room); what `place_sound_object` refuses (its writes put back).

use crate::errors::{BadInput, Refused};
use crate::physics::contact_store::ContactWork;
use crate::physics::step_families::ContactInputs;
use crate::race::owner_course::Course;

use crate::math::next_random::next_random;
use crate::physics::flow::c0;
use crate::physics::place_sound_object::place_sound_object;
use rc2k_core::drive::lit_faces::{LIST_ROOM, Touch, fall_mark, lucky, touch};

/// The usual marks: +9 = 0, +0x11 = `mark`, +0x13 = 0, +0x12 `face_angle`'s low byte.
fn mark(
    course: &Course,
    contact: &ContactWork,
    face: u32,
    mark: u32,
    clear_9: bool,
) -> Result<(), BadInput> {
    let angle = contact.face_angle.get() as u8;
    crate::race::course_places::update_face_number(course, face, |f| {
        f.touch(mark as u8, angle, clear_9);
    })
}

// Converted: `rc2k_core::drive::lit_faces::{touch, fall_mark, lucky}`.
pub fn touch_face(
    c: &ContactInputs<'_>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let number = c.contact.face_number.get();
    let face = number;
    if crate::race::course_places::face_number(c.course, face)?.mark() != 0 {
        return Ok(());
    }
    let count = c.contact.lit_face_count.get();
    if count == LIST_ROOM {
        return Ok(());
    }
    if count > LIST_ROOM {
        return Err(BadInput("g_litFaceCount past 0x100 (the list's room)").into());
    }
    (|| -> Result<(), crate::errors::Refused> {
        c.contact.lit_faces[count as usize].set(number);
        let kind = c.contact.contact_kind.get();
        match touch(kind) {
            Touch::MarkIfFast => {
                if c0(&c.car.speed, &c.numbers.eight) {
                    return Ok(());
                }
                mark(c.course, c.contact, face, 1, true)?;
            }
            Touch::Mark => mark(c.course, c.contact, face, 1, true)?,
            Touch::Fall => mark(
                c.course,
                c.contact,
                face,
                fall_mark(c.contact.face_speed.get()),
                false,
            )?,
            Touch::Chance => {
                let (_, lucky) = lucky(next_random());
                if lucky {
                    crate::race::course_places::update_face_number(c.course, face, |f| {
                        f.set_mark(0xF);
                    })?;
                    c.contact.lit_face_count.set(count + 1);
                }
                return Ok(());
            }
            Touch::Sound => {}
        }
        { place_sound_object(&c.sound, scene) }?;
        c.contact.lit_face_count.set(count + 1);
        Ok(())
    })()
}
