//! The scene objects' tables: the listed faces, the material phase work, the textures listed, and
//! the per-slot tables the objects' shapes are loaded into (first point, point count, normal count,
//! first face, face count, the parts' middles and half extents, the radii).

use crate::memory::cells::ZeroCells;
use crate::memory::session::PlainDword;

#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct ObjectTables {
    /// The faces listed for drawing, no longer used: each object's drawing has its own
    /// (`scene::face_list`). Kept for the store's layout.
    pub listed_faces: [PlainDword; 4096],
    /// The material's length, time and phase; nothing reads them and nothing writes them.
    pub phase_work: [PlainDword; 3],
    /// The textures listed (`scene::list_object_textures`, `scene::load_front_end_objects`).
    pub texture_count: PlainDword,
    /// The listed textures.
    pub texture_list: [PlainDword; 256],
    /// Faces without normals (`scene::add_object_shape`, `scene::load_front_end_objects`).
    pub no_normals: PlainDword,
    /// Each slot's first point.
    pub first_point: [PlainDword; 256],
    /// Each slot's points.
    pub point_counts: [PlainDword; 256],
    /// Each slot's normals.
    pub normal_counts: [PlainDword; 256],
    /// Each slot's first face.
    pub first_face: [PlainDword; 256],
    /// Each slot's faces.
    pub face_counts: [PlainDword; 256],
    /// Six tables of a dword a slot, the middles' x, y, z then the half extents' x, y, z
    /// (`scene::load_object`, `physics::fill_race_slot`).
    pub part_extents: [[PlainDword; 256]; 6],
    /// The middle, half the extent, the lowest and the highest, three dwords each
    /// (`scene::measure_object`).
    pub measures: [PlainDword; 12],
    /// Each slot's reach (`scene::add_object_shape`, `physics::face_contact`).
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
        let tables: [&dyn crate::memory::session::Run; 8] = [
            &self.first_point,
            &self.point_counts,
            &self.normal_counts,
            &self.first_face,
            &self.face_counts,
            &self.part_extents,
            &self.measures,
            &self.radii,
        ];
        crate::memory::session::read_runs(&tables[table as usize..], slot, "an object slot")
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
