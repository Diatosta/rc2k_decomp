//! `scene::list_object_faces` (0x45048C): goes through an object's faces and lists the
//! ones to draw.
//!
//! In:  nothing
//! Out: eax = the last face's number in the table, ecx = the number of faces, ebp = the
//!      last face; everything else preserved
//!
//! The list emptied (`g_listedFaceCount` = 0), `dword_62361C` = -1; then for each of the
//! object's `dword_6350C0` faces, from `dword_6350BC` on in `g_faces`: its number within the
//! object to `dword_623620` and `scene::list_object_face`. One caller, `sub_45045D`. Refuses an
//! object of no faces (the original goes through 2^32) or of more than the list holds, a face's
//! number past the table and a face that is not there.

use crate::errors::Refused;
use crate::race::vars_store::RACE_VARS;
use crate::scene::light_work_store::LIGHT_WORK;
use crate::scene::object_store::OBJECT_WORK;

// ---- game data ----

// ---- Rust version ----

// Converted: `rc2k_core::scene::objects::object_faces`
pub fn list_object_faces(
    list: &mut dyn FnMut(usize) -> Result<(), Refused>,
) -> Result<(), Refused> {
    {
        let (first, count): (u32, u32) = (
            OBJECT_WORK.object_first_face.bits(),
            OBJECT_WORK.object_face_count.bits(),
        );
        // Every face is looked up before anything is written.
        let faces = rc2k_core::scene::objects::object_faces(first, count, |number| {
            crate::scene::objects::face_cell(number as usize).get()
        })?;
        LIGHT_WORK.listed_face_count.set_bits(0u32);
        RACE_VARS.face_mark.set_bits(u32::MAX);
        for (n, &face) in faces.iter().enumerate() {
            RACE_VARS.face_number.set_bits(n as u32);
            list(face as usize)?;
        }
        Ok(())
    }
}
