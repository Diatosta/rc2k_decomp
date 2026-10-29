//! The polygon being worked on, as the functions of `scene` share it: entries of the work
//! table `g_polygonEntries` picked through the list `g_polygonList`
//! (`draw_polygon.rs` has an entry's layout).

use crate::errors::BadInput;
pub use crate::scene::draw_polygon::{ENTRIES, ENTRY_LEN, MOST};
use crate::scene::draw_store::DRAW;
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
    crate::race::checkpoints_store::nth(&crate::scene::polygon_store::POLYGON_WORK.list[0], n)
}

/// The list kept before a clip, cell `n` (as `list_cell`).
pub fn old_list_cell(n: usize) -> &'static Dword {
    crate::race::checkpoints_store::nth(&crate::scene::polygon_store::POLYGON_WORK.old_list[0], n)
}

/// The first record of the pool the clip makes its records in (`g_polygonNextRecord`
/// starts at it).
pub fn pool_start() -> u32 {
    crate::scene::point_records::of_pool(0)
}

/// The textures' records (0x40 bytes each).
pub fn texture_infos_addr() -> u32 {
    crate::scene::polygon_store::POLYGON_WORK.texture_infos[0].words[0].addr()
}

/// The objects' material records (0x80 bytes each).
pub fn materials_addr() -> u32 {
    crate::scene::polygon_store::POLYGON_WORK.object_materials[0].words[0].addr()
}

pub fn dword(at: usize) -> u32 {
    crate::pointers::read_or_end::<u32>((at) as u32, "scene::polygon's operand")
}

/// The polygon's entries in the list's order. Refuses what the originals go wrong on:
/// more vertices than the vertex buffer holds, a list entry past the table.
pub fn listed() -> Result<Vec<&'static PolygonEntry>, BadInput> {
    let count: u32 = DRAW.polygon_vertex_count.bits();
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
