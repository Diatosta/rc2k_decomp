//! Marks and lists the course face touched (`face_number`, in the course faces, 0x40 bytes each). A
//! face whose mark byte is not 0, or a full list (the lit count at 0x100), is left alone.
//! Otherwise its number goes into the lit list's next place and by the contact kind `contact_kind`:
//! - 9 and 1, unless `speed` is below `eight` (or unordered; then nothing more), and 2, 3, 8:
//!   `Face::touch` with the mark 1 and byte 9 cleared;
//! - 4: `Face::touch` with the mark = `face_speed` >> 14 held to 0x40, byte 9 kept;
//! - 0xFF: one in about 85 (`math::next_random` & 0xFF below 3) the mark = 0xF, counted, no sound;
//!   otherwise nothing more;
//! - the rest (10, 5, 6, 7, ...): nothing to the face.
//!
//! Then `physics::place_sound_object` and the count + 1.
//!
//! Refuses (before anything) a count past 0x100 (the list's room); what `place_sound_object`
//! refuses (its writes are put back).

use crate::errors::{BadInput, Refused};
use crate::physics::contact_store::ContactWork;
use crate::physics::step_families::ContactInputs;
use crate::race::owner_course::Course;

use crate::physics::flow::c0;
use crate::physics::place_sound_object::place_sound_object;
use rc2k_core::drive::lit_faces::{LIST_ROOM, Touch, fall_mark, lucky, touch};

/// The usual marks on the face (`Face::touch`): the mark = `mark`, byte 0x13 = 0 and the angle
/// byte = `face_angle`'s low byte; `clear_9` also clears byte 9.
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

// Core: `rc2k_core::drive::lit_faces::{touch, fall_mark, lucky}`.
pub fn touch_face(
    c: &ContactInputs<'_>,
    scene: &crate::scene::owner::Scene,
) -> Result<(), Refused> {
    let number = c.contact.face_number.get();
    let face = number;
    if crate::race::course_places::face_number(c.course.reads(), face)?.mark() != 0 {
        return Ok(());
    }
    let count = c.lit_faces.count.get();
    if count == LIST_ROOM {
        return Ok(());
    }
    if count > LIST_ROOM {
        return Err(BadInput("g_litFaceCount past 0x100 (the list's room)").into());
    }
    (|| -> Result<(), crate::errors::Refused> {
        c.lit_faces.faces[count as usize].set(number);
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
                let (_, lucky) = lucky(c.shared.random.next_random());
                if lucky {
                    crate::race::course_places::update_face_number(c.course, face, |f| {
                        f.set_mark(0xF);
                    })?;
                    c.lit_faces.count.set(count + 1);
                }
                return Ok(());
            }
            Touch::Sound => {}
        }
        { place_sound_object(&c.sound, scene) }?;
        c.lit_faces.count.set(count + 1);
        Ok(())
    })()
}
