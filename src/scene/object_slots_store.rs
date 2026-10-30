//! The scene objects' tables (IDA 0x62DB5C .. 0x634FA0; phase 7, typed): the listed faces,
//! the material phase work, the textures listed, and the per-slot tables the objects' shapes
//! are loaded into (first point, point count, normal count, first face, face count, the
//! parts' middles and half extents, the radii). `#[repr(C)]` with the image's layout, over
//! the race block's bytes (`OBJECT_SLOTS`). Names are readings unless IDA's.

use crate::session::Dword;

#[repr(C)]
pub struct ObjectTables {
    /// +0x0 (0x62DB5C): `g_listedFaces`: the faces listed for drawing, 0x10 bytes each (number, sort key, third word, see-through; `scene::list_object_face`, `scene::sort_listed_faces`, `scene::draw_object_faces`).
    pub listed_faces: [Dword; 4096],
    /// +0x4000 (0x631B5C): `dword_631B5C` .. `dword_631B64`: the material's length, time and phase (`scene::set_object_material`, `scene::step_object_shape`).
    pub phase_work: [Dword; 3],
    /// +0x400c (0x631B68): `dword_631B68`: the textures listed (`scene::list_object_textures`, `scene::load_front_end_objects`).
    pub texture_count: Dword,
    /// +0x4010 (0x631B6C): `dword_631B6C`: the listed textures.
    pub texture_list: [Dword; 256],
    /// +0x4410 (0x631F6C): `dword_631F6C`: faces without normals (`scene::add_object_shape`, `scene::load_front_end_objects`).
    pub no_normals: Dword,
    /// +0x4414 (0x631F70): `g_objectFirstPoint` (and `g_slotFirstPoints` from slot 11): each slot's first point.
    pub first_point: [Dword; 256],
    /// +0x4814 (0x632370): `g_objectPointCount2` (and `g_slotPointCounts` from slot 11): each slot's points.
    pub point_counts: [Dword; 256],
    /// +0x4c14 (0x632770): `g_objectNormalCount`: each slot's normals.
    pub normal_counts: [Dword; 256],
    /// +0x5014 (0x632B70): `dword_632B70`: each slot's first face.
    pub first_face: [Dword; 256],
    /// +0x5414 (0x632F70): `dword_632F70`: each slot's faces.
    pub face_counts: [Dword; 256],
    /// +0x5814 (0x633370): `g_partMiddleX/Y/Z`, `g_partHalfX/Y/Z`: six tables of a dword a slot, the middles' x, y, z then the half extents' x, y, z (`scene::load_object`, `physics::fill_race_slot`).
    pub part_extents: [[Dword; 256]; 6],
    /// +0x7014 (0x634B70): `dword_634B70` .. `dword_634B9C`: the middle, half the extent, the lowest and the highest, three dwords each (`scene::measure_object`).
    pub measures: [Dword; 12],
    /// +0x7044 (0x634BA0): `dword_634BA0`: each slot's reach (`scene::add_object_shape`, `physics::face_contact`).
    pub radii: [Dword; 256],
}

impl ObjectTables {
    /// A slot's entry of one of the 256-entry tables, read by address as the original does:
    /// a slot past the table reads on into the next one (for slots nothing has checked).
    pub fn slot_value(table: &[Dword; 256], slot: usize) -> u32 {
        // The tables are contiguous; a slot far past them is the caller's slip, as it was the
        // original's (a checked read: outside the allocation ends the program).
        crate::pointers::read_or_end(table[0].addr() + (4 * slot) as u32, "an object slot")
    }
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.
unsafe impl crate::pointers::Cells for ObjectTables {}

/// The piece: a static of its own since 2026-10-29 (it left the race block, phase 13:
/// `race::store::LEFT`), with the image's first values.
pub static OBJECT_SLOTS: ObjectTables =
    crate::pointers::cells_of_bytes(crate::race::store::first_values::<{ END - START }>(START));

/// IDA 0x62DB5C, the first byte of the piece.
pub const START: usize = 0x62DB5C;
/// IDA 0x634FA0, the end of the piece.
pub const END: usize = 0x634FA0;

const _: () = assert!(END - START == core::mem::size_of::<ObjectTables>());

/// Where IDA 0x62DB5C is now.
pub fn piece_addr() -> u32 {
    crate::pointers::game_address(core::ptr::from_ref(&OBJECT_SLOTS))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::offset_of;

    #[test]
    fn first_values_are_the_games() {
        crate::gfx::first_values_are_the_games(&OBJECT_SLOTS, START, END);
    }

    #[test]
    fn piece_has_its_home() {
        assert_eq!(piece_addr() % 4, 0);
        assert_eq!(
            crate::moved::place(END - 1),
            (piece_addr() + (END - START) as u32 - 1) as usize
        );
        assert_eq!(offset_of!(ObjectTables, phase_work), 0x631B5C - START);
        assert_eq!(offset_of!(ObjectTables, texture_count), 0x631B68 - START);
        assert_eq!(offset_of!(ObjectTables, texture_list), 0x631B6C - START);
        assert_eq!(offset_of!(ObjectTables, no_normals), 0x631F6C - START);
        assert_eq!(offset_of!(ObjectTables, first_point), 0x631F70 - START);
        assert_eq!(offset_of!(ObjectTables, point_counts), 0x632370 - START);
        assert_eq!(offset_of!(ObjectTables, face_counts), 0x632F70 - START);
        assert_eq!(offset_of!(ObjectTables, part_extents), 0x633370 - START);
        assert_eq!(offset_of!(ObjectTables, measures), 0x634B70 - START);
        assert_eq!(offset_of!(ObjectTables, radii), 0x634BA0 - START);
    }
}
