//! `scene::list_object_faces` (0x45048C): goes through an object's faces and lists the
//! ones to draw.
//!
//! The list emptied (the original's `g_listedFaceCount` = 0), `dword_62361C` = -1; then for each of the
//! object's `dword_6350C0` faces, from `dword_6350BC` on in `g_faces`: its number within the
//! object to `dword_623620` and `scene::list_object_face`. One caller, `scene::draw_object`.
//! Refuses an object of no faces (the original goes through 2^32) or of more than the list holds, a
//! face's number past the table and a face that is not there.

use crate::errors::Refused;
use crate::race::owner_shared::Shared;

// ---- game data ----

// ---- Rust version ----

// Converted: `rc2k_core::scene::objects::object_faces`
pub fn list_object_faces(
    shared: &Shared,
    object: &crate::scene::object_place::ObjectPlace,
    work: &crate::race::face_work::FaceWork,
    faces_list: &mut crate::scene::face_list::FaceList,
    list: &mut dyn FnMut(&mut crate::scene::face_list::FaceList, usize) -> Result<(), Refused>,
) -> Result<(), Refused> {
    {
        let (first, count): (u32, u32) = (object.first_face.bits(), object.face_count.bits());
        // Every face is looked up before anything is written.
        let faces = rc2k_core::scene::objects::object_faces(first, count, |number| {
            crate::scene::objects::face_cell(number as usize).get()
        })?;
        faces_list.clear();
        shared.vars.face_mark.set_bits(u32::MAX);
        for (n, &face) in faces.iter().enumerate() {
            work.face_number.set_bits(n as u32);
            // Read by `scene::log_bad_polygon`.
            shared.vars.face_number.set_bits(n as u32);
            list(faces_list, face as usize)?;
        }
        Ok(())
    }
}
