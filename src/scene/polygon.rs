//! The polygon being worked on, as the functions of `scene` share it: entries of the work
//! table `g_polygonEntries` picked through the list `g_polygonList`
//! (`draw_polygon.rs` has an entry's layout).

use crate::errors::BadInput;
pub use crate::scene::draw_polygon::{ENTRIES, ENTRY_LEN, MOST};
use crate::scene::polygon_store::PolygonEntry;
use crate::session::Dword;

/// The work table's entry `n` (the 64 of the polygon, then the 64 kept for the fan).
/// A number past the table is refused.
pub fn entry(n: usize) -> Result<&'static PolygonEntry, BadInput> {
    crate::scene::polygon_store::POLYGON_WORK
        .entries
        .get(n)
        .ok_or(BadInput("a polygon entry past the work table"))
}

/// The list's cell `n` (the original's list has room for `MOST` numbers, more than the
/// field: the cell is reached by address).
pub fn list_cell(n: usize) -> &'static Dword {
    crate::race::checkpoints_store::cell_in(&crate::scene::polygon_store::POLYGON_WORK.list, n)
}

/// The list kept before a clip, cell `n` (as `list_cell`).
pub fn old_list_cell(n: usize) -> &'static Dword {
    crate::race::checkpoints_store::cell_in(&crate::scene::polygon_store::POLYGON_WORK.old_list, n)
}

/// The first record of the pool the clip makes its records in (`g_polygonNextRecord`
/// starts at it).
pub fn pool_start() -> u32 {
    crate::scene::point_records::of_pool(0)
}

/// The objects' material records (0x80 bytes each), as a pointer to the first bounded by the
/// polygon work around the records, not the
/// records alone: `scene::set_object_material` is given kind -1 and reads the record before
/// the first (the texture infos' tail), as the original does.
pub fn materials_ptr() -> crate::pointers::GamePtr {
    use crate::scene::polygon_store::{POLYGON_TABLES, PolygonTables};
    crate::pointers::GamePtr::to(&*POLYGON_TABLES)
        .offset(core::mem::offset_of!(PolygonTables, object_materials) as u32)
}

/// Byte `offset` of texture info `n` (`g_textureInfos` + 0x40 n): an offset past the
/// record reads on into the next, as the original's arithmetic does.
#[track_caller]
pub fn texture_info_byte(n: u32, offset: u32) -> Result<u8, BadInput> {
    crate::pointers::GamePtr::to(&crate::scene::polygon_store::POLYGON_TABLES.texture_infos)
        .offset(n.wrapping_mul(0x40).wrapping_add(offset))
        .read("a texture info's byte")
}

/// Byte `offset` of object kind `kind`'s material record (`g_objectMaterials` + 0x80 kind;
/// kind -1 the record before the first, as the original reads it).
#[track_caller]
pub fn material_byte(kind: u32, offset: u32) -> Result<u8, BadInput> {
    crate::pointers::GamePtr::to(&crate::scene::polygon_store::POLYGON_TABLES.object_materials)
        .offset(kind.wrapping_mul(0x80).wrapping_add(offset))
        .read("an object's material byte")
}

/// A polygon's entries: on the stack for all but the largest polygons.
pub type Listed = smallvec::SmallVec<[&'static PolygonEntry; 32]>;

/// The polygon's entries in the list's order. Refuses what the originals go wrong on:
/// more vertices than the vertex buffer holds, a list entry past the table.
pub fn listed(poly: &crate::scene::polygon_work::PolygonWork) -> Result<Listed, BadInput> {
    let count: u32 = poly.polygon_vertex_count.bits();
    if count > MOST {
        return Err(BadInput("a polygon of more than 256 vertices"));
    }
    (0..count as usize)
        .map(|n| {
            let number: u32 = crate::scene::polygon::list_cell(n).get();
            if number >= ENTRIES {
                return Err(BadInput("a list entry past the work table"));
            }
            entry(number as usize)
        })
        .collect()
}
