//! The polygon being worked on, as the functions of `scene` share it: entries of the work
//! table `g_polygonEntries` picked through the list `g_polygonList`
//! (`draw_polygon.rs` has an entry's layout).

use crate::d3d::draw_fan_unclipped::COUNT_RVA;
use crate::errors::BadInput;
use crate::framework::{game_addr, game_read};
pub use crate::scene::draw_polygon::{ENTRIES, ENTRIES_RVA, ENTRY_LEN, LIST_RVA, MOST};

pub unsafe fn dword(at: usize) -> u32 {
    unsafe { (at as *const u32).read_unaligned() }
}

pub unsafe fn put(at: usize, value: u32) {
    unsafe { (at as *mut u32).write_unaligned(value) }
}

/// The polygon's entries in the list's order, as addresses. Refuses what the originals
/// go wrong on: more vertices than the vertex buffer holds, a list entry past the table.
pub unsafe fn listed() -> Result<Vec<usize>, BadInput> {
    let count: u32 = unsafe { game_read(COUNT_RVA) };
    if count > MOST {
        return Err(BadInput("a polygon of more than 256 vertices"));
    }
    (0..count as usize)
        .map(|n| {
            let number: u32 = unsafe { game_read(LIST_RVA + n * 4) };
            if number >= ENTRIES {
                return Err(BadInput("a list entry past the work table"));
            }
            Ok(game_addr(ENTRIES_RVA) + number as usize * ENTRY_LEN)
        })
        .collect()
}
