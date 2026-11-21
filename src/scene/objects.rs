//! The tables the objects' shapes are loaded into: shared by the loader's functions.
//!
//! An object slot (0 .. 0xFF) has a first point and a number of points in the points'
//! memory (twelve bytes a point, three whole numbers), its normals after them, and a
//! first face and a number of faces in the faces table, whose entries point into the faces'
//! memory.

pub const SLOTS: u32 = 0x100;
/// The face table's entry `n`. Every caller holds `n` below `FACES` (the object slots' face
/// ranges are checked against it where they are read, and written only by `add_object_shape`
/// within it), so an entry past the table is refused.
pub fn face(faces: crate::scene::faces_store::FacesRef, n: usize) -> u32 {
    faces.read(|table| {
        table.faces.get(n).copied().unwrap_or_else(|| {
            crate::fatal::raise::end(
                "a face number past g_faces",
                crate::errors::BadInput("a face number past g_faces"),
            )
        })
    })
}
/// The face table's entry `n` set (as `face`).
pub fn set_face(race: &crate::race::owner::Race, n: usize, value: u32) {
    let mut table = race.shared.buffers.faces.borrow_mut();
    let Some(entry) = table.faces.get_mut(n) else {
        crate::fatal::raise::end(
            "a face number past g_faces",
            crate::errors::BadInput("a face number past g_faces"),
        )
    };
    *entry = value;
}
pub const FACES: u32 = 0xC000;
/// The file as read: +4 points, +8 faces, +0xC normals, +0x2C the points.
pub fn file(
    race: &crate::race::owner::Race,
) -> &crate::memory::plain_buffer::PlainBuffer<FILE_ROOM> {
    &race.shared.buffers.object_file
}
pub const FILE_ROOM: usize = 0x40000;
