//! The tables the objects' shapes are loaded into: shared by the loader's functions.
//!
//! An object slot (0 .. 0xFF) has a first point and a number of points in the points'
//! memory (twelve bytes a point, three whole numbers), its normals after them, and a
//! first face and a number of faces in `g_faces`, whose entries point into the faces'
//! memory.

pub const SLOTS: u32 = 0x100;
/// The face table's cell `n`. Every caller holds `n` below `FACES` (the object slots' face
/// ranges are checked against it where they are read, and written only by `add_object_shape`
/// within it), so a cell past the table is refused.
pub fn face_cell(faces: &crate::scene::faces_store::Faces, n: usize) -> &crate::session::Dword {
    faces.faces.get(n).unwrap_or_else(|| {
        crate::fatal::raise::end(
            "a face number past g_faces",
            crate::errors::BadInput("a face number past g_faces"),
        )
    })
}
pub const FACES: u32 = 0xC000;
/// The file as read: +4 points, +8 faces, +0xC normals, +0x2C the points.
pub fn file(race: &crate::race::owner::Race) -> &crate::session::Bytes<FILE_ROOM> {
    &race.engine_tables().object_file
}
pub const FILE_ROOM: usize = 0x40000;
