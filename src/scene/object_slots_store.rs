//! The scene objects' tables (IDA 0x62DB5C .. 0x634FA0): the listed faces,
//! the material phase work, the textures listed, and the per-slot tables the objects' shapes
//! are loaded into (first point, point count, normal count, first face, face count, the
//! parts' middles and half extents, the radii). `#[repr(C)]` with the image's layout, over
//! the race block's bytes (`OBJECT_SLOTS`). Names are readings unless IDA's.

use crate::cells::ZeroCells;
use crate::session::PlainDword;

#[repr(C)]
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct ObjectTables {
    /// +0x0 (0x62DB5C): `g_listedFaces`: the original's faces listed for drawing, no longer used: each object's drawing has its own (`scene::face_list`). Kept for the store's layout.
    pub listed_faces: [PlainDword; 4096],
    /// +0x4000 (0x631B5C): `dword_631B5C` .. `dword_631B64`: the material's length, time and phase, which the original's `scene::set_object_material` stored on each call; nothing reads them and nothing writes them.
    pub phase_work: [PlainDword; 3],
    /// +0x400c (0x631B68): `dword_631B68`: the textures listed (`scene::list_object_textures`, `scene::load_front_end_objects`).
    pub texture_count: PlainDword,
    /// +0x4010 (0x631B6C): `dword_631B6C`: the listed textures.
    pub texture_list: [PlainDword; 256],
    /// +0x4410 (0x631F6C): `dword_631F6C`: faces without normals (`scene::add_object_shape`, `scene::load_front_end_objects`).
    pub no_normals: PlainDword,
    /// +0x4414 (0x631F70): `g_objectFirstPoint` (and `g_slotFirstPoints` from slot 11): each slot's first point.
    pub first_point: [PlainDword; 256],
    /// +0x4814 (0x632370): `g_objectPointCount2` (and `g_slotPointCounts` from slot 11): each slot's points.
    pub point_counts: [PlainDword; 256],
    /// +0x4c14 (0x632770): `g_objectNormalCount`: each slot's normals.
    pub normal_counts: [PlainDword; 256],
    /// +0x5014 (0x632B70): `dword_632B70`: each slot's first face.
    pub first_face: [PlainDword; 256],
    /// +0x5414 (0x632F70): `dword_632F70`: each slot's faces.
    pub face_counts: [PlainDword; 256],
    /// +0x5814 (0x633370): `g_partMiddleX/Y/Z`, `g_partHalfX/Y/Z`: six tables of a dword a slot, the middles' x, y, z then the half extents' x, y, z (`scene::load_object`, `physics::fill_race_slot`).
    pub part_extents: [[PlainDword; 256]; 6],
    /// +0x7014 (0x634B70): `dword_634B70` .. `dword_634B9C`: the middle, half the extent, the lowest and the highest, three dwords each (`scene::measure_object`).
    pub measures: [PlainDword; 12],
    /// +0x7044 (0x634BA0): `dword_634BA0`: each slot's reach (`scene::add_object_shape`, `physics::face_contact`).
    pub radii: [PlainDword; 256],
}

impl ObjectTables {
    /// A slot's entry of one of the 256-entry tables, read by address as the original does:
    /// a slot past the table reads on into the next one (for slots nothing has checked).
    pub fn slot_value(&self, table: &[PlainDword; 256], slot: usize) -> u32 {
        // The tables are contiguous; a slot far past them is the caller's slip, as it was the
        // original's (a checked read: outside the allocation ends the program).
        crate::session::read_on_in(self, table, slot, "an object slot")
            .unwrap_or_else(|e| crate::fatal::raise::end("an object slot", e))
    }
}

// SAFETY: every field is a cell over a plain integer (`session`'s cells and records of
// them): any bit pattern is a valid value.

impl ObjectTables {
    /// The store with its first values (every field not named here starts at zero).
    pub const fn new() -> Self {
        <Self as ZeroCells>::ZERO
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn first_values_are_the_games() {
        crate::test_support::first_values::assert_block_starts_as_the_games(
            crate::scene::owner::scene().objects.slots.get(),
        );
    }
}
