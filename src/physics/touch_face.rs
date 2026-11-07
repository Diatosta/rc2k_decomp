//! `physics::touch_face` (0x45E9E0): the course face touched (`dword_6D3218`, in
//! `g_courseFaces`, 0x40 bytes each) marked and listed. A face whose byte +0x11 is not 0,
//! or a full list (`g_litFaceCount` at 0x100), is left alone. Else its number goes into
//! `g_litFaces[count]` and by the contact kind `dword_6D3624`:
//! - 9 and 1, unless `unk_71A19C` is below `flt_5C5D50` (or unordered; then nothing more),
//!   and 2, 3, 8: bytes +9 = 0, +0x11 = 1, +0x13 = 0, +0x12 = `dword_6D3B70`'s low byte;
//! - 4: +0x11 = `dword_6D3B74` >> 14 held to 0x40, +0x13 = 0, +0x12 as above;
//! - 0xFF: one in about 85 (`math::next_random` & 0xFF below 3) +0x11 = 0xF, counted, no
//!   sound; else nothing more;
//! - the rest (10, 5, 6, 7, ...): nothing to the face.
//!
//! Then `physics::place_sound_object` and the count + 1.
//!
//! In:  nothing
//! Out: ebx the face's number, esi its address; when the face was free, edx the count
//!      before; eax as the kind's path leaves it (the kind, the random number, or
//!      `dword_6D3B70`); the rest kept; no flags read (callers return)
//!
//! Callers: `sub_45E3BF` (six). Refuses (before anything) a count past 0x100 (the list's
//! room); what `place_sound_object` refuses (its writes put back).

use crate::errors::{BadInput, Refused};
use crate::race::owner::Race;
use crate::race::owner_physics::Physics;

use crate::math::next_random::next_random;
use crate::physics::flow::{c0, p};
use crate::physics::place_sound_object::place_sound_object;
use rc2k_core::drive::lit_faces::{LIST_ROOM, Touch, fall_mark, lucky, touch};

/// The usual marks: +9 = 0, +0x11 = `mark`, +0x13 = 0, +0x12 `dword_6D3B70`'s low byte.
fn mark(physics: &Physics, face: u32, mark: u32, clear_9: bool) -> Result<(), BadInput> {
    let angle = physics.contact.face_angle.get() as u8;
    crate::race::course_places::update_face_number(face, "a touched face", |f| {
        f.touch(mark as u8, angle, clear_9);
    })
}

/// The Rust version.
// Converted: `rc2k_core::drive::lit_faces::{touch, fall_mark, lucky}`.
pub fn touch_face(race: &Race) -> Result<(), Refused> {
    let scene = crate::scene::owner::scene();
    let race_owner = race;
    let number = race_owner.physics.contact.face_number.get();
    let face = number;
    if crate::race::course_places::face_number(face, "a touched face")?.mark() != 0 {
        return Ok(());
    }
    let count = race_owner.physics.contact.lit_face_count.get();
    if count == LIST_ROOM {
        return Ok(());
    }
    if count > LIST_ROOM {
        return Err(BadInput("g_litFaceCount past 0x100 (the list's room)").into());
    }
    (|| -> Result<(), crate::errors::Refused> {
        p(
            &race_owner.physics.contact.lit_faces[count as usize],
            number,
        );
        let kind = race_owner.physics.contact.contact_kind.get();
        match touch(kind) {
            Touch::MarkIfFast => {
                if c0(
                    race.shared.car.speed.bits(),
                    race_owner.course.numbers.eight.bits(),
                ) {
                    return Ok(());
                }
                mark(&race.physics, face, 1, true)?;
            }
            Touch::Mark => mark(&race.physics, face, 1, true)?,
            Touch::Fall => mark(
                &race.physics,
                face,
                fall_mark(race_owner.physics.contact.face_speed.get()),
                false,
            )?,
            Touch::Chance => {
                let (_, lucky) = lucky(next_random());
                if lucky {
                    crate::race::course_places::update_face_number(face, "a touched face", |f| {
                        f.set_mark(0xF);
                    })?;
                    race_owner.physics.contact.lit_face_count.set(count + 1);
                }
                return Ok(());
            }
            Touch::Sound => {}
        }
        { place_sound_object(race, scene) }?;
        race_owner.physics.contact.lit_face_count.set(count + 1);
        Ok(())
    })()
}
