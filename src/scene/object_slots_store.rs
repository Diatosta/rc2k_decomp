//! The scene objects' per-slot tables. `SlotShapes` (each slot's first point, point count, normal
//! count, first face and face count) is what the drawing parts read, a plain record lent as a copy; `LoadedSlots`
//! (the textures listed, the parts' middles and half extents, the radii) is plain fields only the
//! game's thread reaches (`scene::owner::Scene::loaded_slots`).

use crate::memory::session::{DrawPlain, PlainRead};

/// What the objects' shapes are loaded as, by slot (`scene::add_object_shape`,
/// `scene::reset_objects`): a plain record (`session::DrawPlain`) written on the game's thread,
/// read there and, as a copy made when a drawing scope opens, by the drawing parts through their
/// reads (`scene::workers::DrawReads`).
#[derive(Clone)]
pub struct SlotShapes {
    /// Each slot's first point.
    pub first_point: [u32; 256],
    /// Each slot's points.
    pub point_counts: [u32; 256],
    /// Each slot's normals.
    pub normal_counts: [u32; 256],
    /// Each slot's first face.
    pub first_face: [u32; 256],
    /// Each slot's faces.
    pub face_counts: [u32; 256],
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
            Some(&value) => value,
            None => crate::memory::host_hooks::end(
                "an object slot",
                crate::errors::BadInput("an object slot past the tables"),
            ),
        }
    }

    /// The store with its first values (everything starts at zero).
    pub const fn new() -> Self {
        Self {
            first_point: [0; 256],
            point_counts: [0; 256],
            normal_counts: [0; 256],
            first_face: [0; 256],
            face_counts: [0; 256],
        }
    }
}

/// The slots as one reader finds them: the game's thread's own (read through the store when used),
/// or a drawing part's copy.
#[derive(Clone, Copy)]
pub enum SlotShapesRef<'a> {
    Store(&'a DrawPlain<SlotShapes>),
    Copy(&'a SlotShapes),
}

impl SlotShapesRef<'_> {
    /// Slot `slot`'s first point.
    pub fn first_point(self, slot: usize) -> u32 {
        match self {
            Self::Store(store) => store.borrow().first_point[slot],
            Self::Copy(shapes) => shapes.first_point[slot],
        }
    }

    /// Slot `slot`'s point count.
    pub fn point_count(self, slot: usize) -> u32 {
        match self {
            Self::Store(store) => store.borrow().point_counts[slot],
            Self::Copy(shapes) => shapes.point_counts[slot],
        }
    }

    /// Slot `slot`'s normal count.
    pub fn normal_count(self, slot: usize) -> u32 {
        match self {
            Self::Store(store) => store.borrow().normal_counts[slot],
            Self::Copy(shapes) => shapes.normal_counts[slot],
        }
    }

    /// Slot `slot`'s first face.
    pub fn first_face(self, slot: usize) -> u32 {
        match self {
            Self::Store(store) => store.borrow().first_face[slot],
            Self::Copy(shapes) => shapes.first_face[slot],
        }
    }

    /// Slot `slot`'s face count.
    pub fn face_count(self, slot: usize) -> u32 {
        match self {
            Self::Store(store) => store.borrow().face_counts[slot],
            Self::Copy(shapes) => shapes.face_counts[slot],
        }
    }
}

impl<'a> From<&'a PlainRead<SlotShapes>> for SlotShapesRef<'a> {
    fn from(copy: &'a PlainRead<SlotShapes>) -> Self {
        Self::Copy(copy)
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
        let mut t = Box::new(SlotShapes::new());
        t.first_point[7] = 1;
        t.point_counts[7] = 2;
        t.first_face[7] = 3;
        t.face_counts[7] = 4;
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
