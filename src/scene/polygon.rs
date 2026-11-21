//! The polygon being worked on, as the functions of `scene` share it: entries of the work table
//! picked through the list (`draw_polygon.rs` has an entry's layout).

use crate::errors::BadInput;
use crate::memory::session::PlainDword;
pub use crate::scene::draw_polygon::{ENTRIES, MOST};
use crate::scene::polygon_store::{PolygonEntry, PolygonWork};

/// The work table's entry `n` (the 64 of the polygon, then the 64 kept for the fan) in the
/// drawing part's work. A number past the table is refused.
pub fn entry(work: &PolygonWork, n: usize) -> Result<&PolygonEntry, BadInput> {
    work.entries
        .get(n)
        .ok_or(BadInput("a polygon entry past the work table"))
}

/// The list's cell `n`. The list has room for 64 numbers, and a polygon of more vertices (up to
/// `MOST`, or a clip's up to twice that) goes on into the old list's place and the gap after
/// it, which the old list's cells share.
pub fn list_cell(work: &PolygonWork, n: usize) -> &PlainDword {
    let after_list = n.wrapping_sub(work.list.len());
    let after_old = after_list.wrapping_sub(work.old_list.len());
    work.list
        .get(n)
        .or_else(|| work.old_list.get(after_list))
        .or_else(|| work.reserved_2400.get(after_old))
        .unwrap_or_else(|| {
            crate::fatal::raise::end(
                "a polygon list cell past the lists",
                BadInput("a polygon list cell past the lists"),
            )
        })
}

/// The list kept before a clip, cell `n`: the list's cell 64 + `n` (as `list_cell`).
pub fn old_list_cell(work: &PolygonWork, n: usize) -> &PlainDword {
    list_cell(work, work.list.len() + n)
}

/// The first record of the pool the clip makes its records in (`polygon_next_record` starts at
/// it).
pub fn pool_start() -> u32 {
    crate::scene::point_records::of_pool(0)
}

/// The `bytes` from byte `offset` of object kind `kind`'s material record (0x80 bytes each),
/// read with the bound of the polygon tables around the records, not the records alone:
/// `scene::set_object_material` is given kind -1 and reads the record before the first (the
/// texture infos' tail), and a read can run on into the next record. A read past the tables is
/// refused.
pub fn material_run(
    tables: crate::scene::polygon_store::PolygonTablesRef,
    kind: u32,
    offset: u32,
    bytes: &mut [u8],
) -> Result<(), BadInput> {
    // The materials follow the texture infos, so kind -1 is the infos' last record.
    let at = (256 * 0x40u32)
        .wrapping_add(kind.wrapping_mul(0x80))
        .wrapping_add(offset);
    tables.read(|t| t.read_bytes(at as usize, bytes, "a material record's bytes"))
}

/// The byte at `offset` of object kind `kind`'s material record (as `material_run`).
pub fn material_run_byte(
    tables: crate::scene::polygon_store::PolygonTablesRef,
    kind: u32,
    offset: u32,
) -> Result<u8, BadInput> {
    let mut byte = [0u8];
    material_run(tables, kind, offset, &mut byte)?;
    Ok(byte[0])
}

/// The little-endian word at `offset` of object kind `kind`'s material record (as
/// `material_run`).
pub fn material_run_word(
    tables: crate::scene::polygon_store::PolygonTablesRef,
    kind: u32,
    offset: u32,
) -> Result<u16, BadInput> {
    let mut word = [0u8; 2];
    material_run(tables, kind, offset, &mut word)?;
    Ok(u16::from_le_bytes(word))
}

/// Byte `offset` of texture info `n`: an offset past the record reads on into the next.
pub fn texture_info_byte(
    tables: crate::scene::polygon_store::PolygonTablesRef,
    n: u32,
    offset: u32,
) -> Result<u8, BadInput> {
    let what = "a texture info's byte past the table";
    let mut byte = [0u8];
    tables
        .read(|t| {
            t.read_info_bytes(
                n.wrapping_mul(0x40).wrapping_add(offset) as usize,
                &mut byte,
                what,
            )
        })
        .map_err(|_| BadInput(what))?;
    Ok(byte[0])
}

/// A polygon's entries: on the stack for all but the largest polygons.
pub type Listed<'w> = smallvec::SmallVec<[&'w PolygonEntry; 32]>;

/// The polygon's entries in the list's order. Refuses more vertices than the vertex buffer
/// holds and a list entry past the table.
pub fn listed<'w>(
    work: &'w PolygonWork,
    poly: &crate::scene::polygon_work_store::PolygonWork,
) -> Result<Listed<'w>, BadInput> {
    let count: u32 = poly.scalars.borrow().polygon_vertex_count;
    if count > MOST {
        return Err(BadInput("a polygon of more than 256 vertices"));
    }
    (0..count as usize)
        .map(|n| {
            let number: u32 = list_cell(work, n).get();
            if number >= ENTRIES {
                return Err(BadInput("a list entry past the work table"));
            }
            entry(work, number as usize)
        })
        .collect()
}

#[cfg(test)]
mod tests;
