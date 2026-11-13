//! The scene objects' tables (IDA 0x62DB5C .. 0x634FA0): the listed faces,
//! the material phase work, the textures listed, and the per-slot tables the objects' shapes
//! are loaded into (first point, point count, normal count, first face, face count, the
//! parts' middles and half extents, the radii). Names are readings unless IDA's.

use crate::cells::ZeroCells;
use crate::session::PlainDword;

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

/// The per-slot tables the effects read by slot (`ObjectTables::slot_value`), numbered by their
/// place among the tables from `first_point` on.
#[derive(Clone, Copy)]
pub enum SlotTable {
    FirstPoint = 0,
    PointCounts = 1,
    FirstFace = 3,
    FaceCounts = 4,
}

impl ObjectTables {
    /// A slot's entry of one of the 256-entry tables. The slots are not all checked against the
    /// table: a slot past it reads on into the tables after it, in order, to the end of the
    /// record (a slot past that ends the program).
    pub fn slot_value(&self, table: SlotTable, slot: usize) -> u32 {
        let own = match table {
            SlotTable::FirstPoint => &self.first_point,
            SlotTable::PointCounts => &self.point_counts,
            SlotTable::FirstFace => &self.first_face,
            SlotTable::FaceCounts => &self.face_counts,
        };
        if let Some(cell) = own.get(slot) {
            return cell.get();
        }
        let tables: [&dyn crate::session::Run; 8] = [
            &self.first_point,
            &self.point_counts,
            &self.normal_counts,
            &self.first_face,
            &self.face_counts,
            &self.part_extents,
            &self.measures,
            &self.radii,
        ];
        crate::session::read_runs(&tables[table as usize..], slot, "an object slot")
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
    use super::*;

    /// Every slot from each table to the end of the record gives the entry of the tables after it.
    #[test]
    fn slot_value_runs_on_through_the_tables_after() {
        let t = Box::new(ObjectTables::new());
        let parts: [&[PlainDword]; 8] = [
            &t.first_point,
            &t.point_counts,
            &t.normal_counts,
            &t.first_face,
            &t.face_counts,
            t.part_extents.as_flattened(),
            &t.measures,
            &t.radii,
        ];
        let chain: Vec<&PlainDword> = parts.into_iter().flatten().collect();
        for (n, cell) in chain.iter().enumerate() {
            cell.set(0x5000 + n as u32);
        }
        for (table, first) in [
            (SlotTable::FirstPoint, 0),
            (SlotTable::PointCounts, 256),
            (SlotTable::FirstFace, 3 * 256),
            (SlotTable::FaceCounts, 4 * 256),
        ] {
            let left = chain.len() - first;
            for n in (0..left).step_by(7).chain(left - 3..left) {
                assert_eq!(
                    t.slot_value(table, n),
                    0x5000 + (first + n) as u32,
                    "slot {n}"
                );
            }
        }
    }
}
