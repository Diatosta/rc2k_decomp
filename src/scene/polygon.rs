//! The polygon being worked on, as the functions of `scene` share it: entries of the work
//! table `g_polygonEntries` picked through the list `g_polygonList`
//! (`draw_polygon.rs` has an entry's layout).

use crate::errors::BadInput;
pub use crate::scene::draw_polygon::{ENTRIES, ENTRY_LEN, MOST};
use crate::scene::draw_store::DRAW;
use crate::session::Dword;

/// The work table's first entry's address.
pub fn entries_addr() -> usize {
    crate::scene::polygon_store::POLYGON_WORK.entries[0]
        .point
        .addr()
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

/// The pool of point records the clip makes.
pub fn pool_addr() -> usize {
    crate::scene::polygon_store::POLYGON_WORK.record_pool[0].addr()
}

/// The textures' records (0x40 bytes each).
pub fn texture_infos_addr() -> usize {
    crate::scene::polygon_store::POLYGON_WORK.texture_infos[0].words[0].addr()
}

/// The objects' material records (0x80 bytes each).
pub fn materials_addr() -> usize {
    crate::scene::polygon_store::POLYGON_WORK.object_materials[0].words[0].addr()
}

pub fn dword(at: usize) -> u32 {
    crate::pointers::read_or_end::<u32>((at) as u32, "scene::polygon's operand")
}

pub fn put(at: usize, value: u32) {
    crate::pointers::write_or_end::<u32>((at) as u32, value, "scene::polygon's operand")
}

/// The polygon's entries in the list's order, as addresses. Refuses what the originals
/// go wrong on: more vertices than the vertex buffer holds, a list entry past the table.
pub fn listed() -> Result<Vec<usize>, BadInput> {
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
            Ok(crate::scene::polygon::entries_addr() + number as usize * ENTRY_LEN)
        })
        .collect()
}
