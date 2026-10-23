//! `scene::list_object_faces` (0x45048C): goes through an object's faces and lists the
//! ones to draw.
//!
//! In:  nothing
//! Out: eax = the last face's number in the table, ecx = the number of faces, ebp = the
//!      last face; everything else preserved
//!
//! The list emptied (`g_listedFaceCount` = 0), `dword_62361C` = -1; then for each of
//! the object's `dword_6350C0` faces, from `dword_6350BC` on in `g_faces`: its number
//! within the object to `dword_623620` and `scene::list_object_face`.
//! One caller, `sub_45045D`.
//! Refuses an object of no faces (the original goes through 2^32) or of more than the
//! list holds, a face's number past the table and a face that is not there.
//! Shadow mode runs the original alone (each face is compared in
//! `scene::list_object_face`'s hook).
//! Self-test: a redirected copy whose call notes the face and its number.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::{EAX, EBP, ECX, game_read, game_write, rva};
use crate::scene::draw_object_faces::LISTED_RVA;
use crate::screens::harness::Left;

// ---- game data ----

const MARK_RVA: usize = rva(0x62361C); // dword_62361C
const FACE_NUMBER_RVA: usize = rva(0x623620); // dword_623620
const FIRST_FACE_RVA: usize = rva(0x6350BC); // dword_6350BC
const FACE_COUNT_RVA: usize = rva(0x6350C0); // dword_6350C0
const FACES_RVA: usize = rva(0x9A3730); // g_faces

// ---- Rust version ----

// Converted: `rc2k_core::scene::objects::object_faces`
pub unsafe fn list_object_faces(
    list: &mut dyn FnMut(usize) -> Result<(), Refused>,
) -> Result<Left, Refused> {
    unsafe {
        let (first, count): (u32, u32) = (game_read(FIRST_FACE_RVA), game_read(FACE_COUNT_RVA));
        // Every face is looked up before anything is written.
        let faces = rc2k_core::scene::objects::object_faces(first, count, |number| {
            game_read(FACES_RVA + number as usize * 4)
        })?;
        game_write(LISTED_RVA, 0u32);
        game_write(MARK_RVA, u32::MAX);
        for (n, &face) in faces.iter().enumerate() {
            game_write(FACE_NUMBER_RVA, n as u32);
            list(face as usize)?;
        }
        Ok(Left::nothing()
            .with(EAX, (count - 1).wrapping_add(first))
            .with(ECX, count)
            .with(EBP, faces[count as usize - 1]))
    }
}
