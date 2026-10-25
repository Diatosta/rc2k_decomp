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
use crate::physics::car_store::CAR;
use crate::physics::contact_store::CONTACT_WORK;
use crate::race::course_store::COURSE;
use crate::race::numbers_store::NUMBERS;

use crate::math::next_random::next_random;
use crate::physics::flow::{a, c0, g, p};
use crate::physics::place_sound_object::place_sound_object;
use rc2k_core::drive::lit_faces::{LIST_ROOM, Touch, fall_mark, lucky, touch};

/// `g_courseFaces`: the pointer to the course's faces (still named by other files).
pub const FACES: usize = 0x5CCCBC;
const KIND: usize = crate::physics::point_hits_car::KIND;

fn put(at: u32, v: u32) {
    unsafe { (at as *mut u8).write(v as u8) };
}

/// The usual marks: +9 = 0, +0x11 = `mark`, +0x13 = 0, +0x12 `dword_6D3B70`'s low byte.
fn mark(face: u32, mark: u32, clear_9: bool) {
    if clear_9 {
        put(face + 9, 0);
    }
    put(face + 0x11, mark);
    put(face + 0x13, 0);
    let low = CONTACT_WORK.face_angle.get();
    put(face + 0x12, low);
}

/// The Rust version.
// Converted: `rc2k_core::drive::lit_faces::{touch, fall_mark, lucky}`.
pub unsafe fn touch_face() -> Result<(), Refused> {
    let number = CONTACT_WORK.face_number.get();
    let face = (number << 6).wrapping_add(g(&COURSE.course_faces));
    if unsafe { (face as *const u8).add(0x11).read() } != 0 {
        return Ok(());
    }
    let count = CONTACT_WORK.lit_face_count.get();
    if count == LIST_ROOM {
        return Ok(());
    }
    if count > LIST_ROOM {
        return Err(BadInput("g_litFaceCount past 0x100 (the list's room)").into());
    }
    let _all = [
        (a(&CONTACT_WORK.lit_faces[count as usize]), 4),
        (face, 0x40),
        ((CONTACT_WORK.lit_face_count.addr() as u32), 4),
    ];
    (|| -> Result<(), crate::errors::Refused> {
        p(&CONTACT_WORK.lit_faces[count as usize], number);
        let kind = g(KIND);
        match touch(kind) {
            Touch::MarkIfFast => {
                if c0(CAR.speed.bits(), NUMBERS.eight.bits()) {
                    return Ok(());
                }
                mark(face, 1, true);
            }
            Touch::Mark => mark(face, 1, true),
            Touch::Fall => mark(face, fall_mark(CONTACT_WORK.face_speed.get()), false),
            Touch::Chance => {
                let (_, lucky) = lucky(unsafe { next_random() });
                if lucky {
                    put(face + 0x11, 0xF);
                    CONTACT_WORK.lit_face_count.set(count + 1);
                }
                return Ok(());
            }
            Touch::Sound => {}
        }
        unsafe { place_sound_object() }?;
        CONTACT_WORK.lit_face_count.set(count + 1);
        Ok(())
    })()
}
