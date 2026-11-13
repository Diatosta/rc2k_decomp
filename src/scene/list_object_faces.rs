//! `scene::list_object_faces` (0x45048C): goes through an object's faces and lists the
//! ones to draw.
//!
//! The list emptied (the original's `g_listedFaceCount` = 0; its -1 into `RaceVars.face_mark`, read by nothing, not kept); then for each of the
//! object's `ObjectWork.object_face_count` faces, from `ObjectWork.object_first_face` on in `g_faces`: its number within the
//! object to `RaceVars.face_number` and `scene::list_object_face`. One caller, `scene::draw_object`.
//! Refuses an object of no faces (the original goes through 2^32) or of more than the list holds, a
//! face's number past the table and a face that is not there.

use crate::errors::Refused;

// ---- game data ----

// ---- Rust version ----

// Converted: `rc2k_core::scene::objects::object_faces`
pub fn list_object_faces(
    faces_table: &crate::scene::faces_store::Faces,
    object: &crate::scene::object_place::ObjectPlace,
    work: &crate::race::face_work::FaceWork,
    faces_list: &mut crate::scene::face_list::FaceList,
    list: &mut dyn FnMut(&mut crate::scene::face_list::FaceList, usize) -> Result<(), Refused>,
) -> Result<(), Refused> {
    {
        let (first, count): (u32, u32) = (object.first_face.get(), object.face_count.get());
        // Every face is looked up before anything is written.
        let faces = rc2k_core::scene::objects::object_faces(first, count, |number| {
            crate::scene::objects::face_cell(faces_table, number as usize).get()
        })?;
        faces_list.clear();
        // Room for every face at once: the list grows from nothing for each object otherwise.
        faces_list.reserve(faces.len());
        // (The original's -1 into `g_faceMark`, 0x62361C: read by nothing.)
        for (n, &face) in faces.iter().enumerate() {
            work.face_number.set(n as u32);
            list(faces_list, face as usize)?;
        }
        Ok(())
    }
}
