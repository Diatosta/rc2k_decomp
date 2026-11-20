//! The scene objects' per-slot tables. `SlotShapes` (each slot's first point, point count, normal
//! count, first face and face count) is what the drawing parts read, a lent store; `LoadedSlots`
//! (the textures listed, the parts' middles and half extents, the radii) is plain fields only the
//! game's thread reaches (`scene::owner::Scene::loaded_slots`).

use crate::memory::cells::ZeroCells;
use crate::memory::session::PlainDword;

/// What the objects' shapes are loaded as, by slot (`scene::add_object_shape`,
/// `scene::reset_objects`), read by the drawing parts through their reads
/// (`scene::workers::DrawReads`).
#[derive(rc2k_derive::Cells, rc2k_derive::ZeroCells)]
pub struct SlotShapes {
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
}

/// The per-slot tables the effects read by slot (`SlotShapes::slot_value`).
#[derive(Clone, Copy)]
pub enum SlotTable {
    FirstPoint,
    PointCounts,
    FirstFace,
    FaceCounts,
}

impl SlotShapes {
    /// A slot's entry of one of the 256-entry tables (a slot past the table ends the program).
    pub fn slot_value(&self, table: SlotTable, slot: usize) -> u32 {
        let own = match table {
            SlotTable::FirstPoint => &self.first_point,
            SlotTable::PointCounts => &self.point_counts,
            SlotTable::FirstFace => &self.first_face,
            SlotTable::FaceCounts => &self.face_counts,
        };
        match own.get(slot) {
            Some(cell) => cell.get(),
            None => crate::memory::host_hooks::end(
                "an object slot",
                crate::errors::BadInput("an object slot past the tables"),
            ),
        }
    }

    /// The store with its first values (everything starts at zero).
    pub const fn new() -> Self {
        <Self as ZeroCells>::ZERO
    }
}

/// The slots' other tables: plain fields, reached by the game's thread alone.
#[derive(Clone)]
pub struct LoadedSlots {
    /// The textures listed (`scene::list_object_textures`, `scene::load_front_end_objects`).
    pub texture_count: u32,
    /// The listed textures.
    pub texture_list: [u32; 256],
    /// Faces without normals (`scene::add_object_shape`, `scene::load_front_end_objects`).
    pub no_normals: u32,
    /// Six tables of a dword a slot, the middles' x, y, z then the half extents' x, y, z
    /// (`scene::load_object`, `physics::fill_race_slot`).
    pub part_extents: [[u32; 256]; 6],
    /// The middle, half the extent, the lowest and the highest, three dwords each
    /// (`scene::measure_object`).
    pub measures: [u32; 12],
    /// Each slot's reach (`scene::add_object_shape`, `physics::face_contact`).
    pub radii: [u32; 256],
}

impl LoadedSlots {
    /// The tables with their first values (everything starts at zero).
    pub const fn new() -> Self {
        Self {
            texture_count: 0,
            texture_list: [0; 256],
            no_normals: 0,
            part_extents: [[0; 256]; 6],
            measures: [0; 12],
            radii: [0; 256],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each table answers by slot.
    #[test]
    fn slot_value_reads_the_table_asked_for() {
        let t = Box::new(SlotShapes::new());
        t.first_point[7].set(1);
        t.point_counts[7].set(2);
        t.first_face[7].set(3);
        t.face_counts[7].set(4);
        for (table, expected) in [
            (SlotTable::FirstPoint, 1),
            (SlotTable::PointCounts, 2),
            (SlotTable::FirstFace, 3),
            (SlotTable::FaceCounts, 4),
        ] {
            assert_eq!(t.slot_value(table, 7), expected);
        }
    }
}
