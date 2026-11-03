//! The tables the objects' shapes are loaded into: shared by the loader's functions.
//!
//! An object slot (0 .. 0xFF) has a first point and a number of points in the points'
//! memory (twelve bytes a point, three whole numbers), its normals after them, and a
//! first face and a number of faces in `g_faces`, whose entries point into the faces'
//! memory.

pub const SLOTS: u32 = 0x100;
/// The face table's cell `n` (the callers hold `n` to `FACES`, but for what the original
/// leaves unchecked).
pub fn face_cell(n: usize) -> &'static crate::session::Dword {
    crate::race::checkpoints_store::cell_in(&crate::scene::faces_store::FACES.faces, n)
}
pub const FACES: u32 = 0xC000;
/// The file as read: +4 points, +8 faces, +0xC normals, +0x2C the points.
pub fn file() -> &'static crate::session::Bytes<FILE_ROOM> {
    &crate::scene::engine_tables_store::ENGINE_TABLES.object_file
}
pub const FILE_ROOM: usize = 0x40000;
