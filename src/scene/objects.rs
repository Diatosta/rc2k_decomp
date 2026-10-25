//! The tables the objects' shapes are loaded into: shared by the loader's functions.
//!
//! An object slot (0 .. 0xFF) has a first point and a number of points in the points'
//! memory (twelve bytes a point, three whole numbers), its normals after them, and a
//! first face and a number of faces in `g_faces`, whose entries point into the faces'
//! memory.

use crate::framework::{game_read, rva};

pub const SLOTS: u32 = 0x100;
pub const POINT_COUNT_RVA: usize = rva(0x5CD680); // g_objectPointCount
pub const FACE_COUNT_RVA: usize = rva(0x5CD67C); // dword_5CD67C
pub const FACES_RVA: usize = rva(0x9A3730); // g_faces
pub const FACES: u32 = 0xC000;
pub const FIRST_POINT_RVA: usize = rva(0x631F70); // g_objectFirstPoint
pub const POINT_COUNTS_RVA: usize = rva(0x632370); // g_objectPointCount2
pub const NORMAL_COUNTS_RVA: usize = rva(0x632770); // g_objectNormalCount
pub const FIRST_FACE_RVA: usize = rva(0x632B70); // dword_632B70
pub const FACE_COUNTS_RVA: usize = rva(0x632F70); // dword_632F70
pub const OBJECT_KINDS_RVA: usize = rva(0x5CD27C); // g_objectKinds
/// The file as read: +4 points, +8 faces, +0xC normals, +0x2C the points.
pub const FILE_RVA: usize = rva(0xA2BB54); // byte_A2BB54
pub const FILE_ROOM: usize = 0x40000;

pub unsafe fn of_slot(table: usize, slot: u32) -> u32 {
    unsafe { game_read(table + slot as usize * 4) }
}
