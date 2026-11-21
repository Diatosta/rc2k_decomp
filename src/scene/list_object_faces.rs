//! `scene::list_object_faces`: goes through an object's faces and lists the ones to draw.
//!
//! The list is emptied; then for each of the object's `ObjectWork.object_face_count` faces, from
//! `ObjectWork.object_first_face` on in the faces table: its number within the object to
//! `RaceVars.face_number` and `scene::list_object_face`. Refuses an object of no faces or of more
//! than the list holds, a face's number past the table and a face that is not there.

use crate::errors::Refused;

// ---- game data ----

// Logic: `rc2k_core::scene::objects::object_faces`
pub fn list_object_faces(
    faces_table: &crate::scene::faces_store::Faces,
    object: &crate::scene::object_place::ObjectPlace,
    work: &crate::race::face_work::FaceWork,
    faces_list: &mut crate::scene::face_list::FaceList,
    list: &mut dyn FnMut(&mut crate::scene::face_list::FaceList, usize) -> Result<(), Refused>,
) -> Result<(), Refused> {
    {
        let (first, count): (u32, u32) = (object.borrow().first_face, object.borrow().face_count);
        // Every face is looked up before anything is written.
        let faces = rc2k_core::scene::objects::object_faces(first, count, |number| {
            crate::scene::objects::face_cell(faces_table, number as usize).get()
        })?;
        faces_list.clear();
        // Room for every face at once: the list grows from nothing for each object otherwise.
        faces_list.reserve(faces.len());
        for (n, &face) in faces.iter().enumerate() {
            work.borrow_mut().face_number = n as u32;
            list(faces_list, face as usize)?;
        }
        Ok(())
    }
}
