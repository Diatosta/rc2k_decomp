//! The polygon being worked on, as the functions of `scene` share it: entries of the work
//! table `g_polygonEntries` picked through the list `g_polygonList`
//! (`draw_polygon.rs` has an entry's layout).

use crate::errors::BadInput;
pub use crate::scene::draw_polygon::{ENTRIES, ENTRY_LEN, MOST};
use crate::scene::polygon_store::{PolygonEntry, PolygonWork};
use crate::session::Dword;

/// The work table's entry `n` (the 64 of the polygon, then the 64 kept for the fan) in the
/// drawing part's work. A number past the table is refused.
pub fn entry(work: &PolygonWork, n: usize) -> Result<&PolygonEntry, BadInput> {
    work.entries
        .get(n)
        .ok_or(BadInput("a polygon entry past the work table"))
}

/// The list's cell `n`. The original keeps the list with room for 64 numbers, and a
/// polygon of more vertices (up to `MOST`, or a clip's up to twice that) goes on into the
/// old list's place and the gap after it, which the old list's cells share.
pub fn list_cell(work: &PolygonWork, n: usize) -> &Dword {
    let after_list = n.wrapping_sub(work.list.len());
    let after_old = after_list.wrapping_sub(work.old_list.len());
    work.list
        .get(n)
        .or_else(|| work.old_list.get(after_list))
        .or_else(|| work.unknown_2400.get(after_old))
        .unwrap_or_else(|| {
            crate::fatal::raise::end(
                "a polygon list cell past the lists",
                BadInput("a polygon list cell past the lists"),
            )
        })
}

/// The list kept before a clip, cell `n`: the list's cell 64 + `n` (as `list_cell`).
pub fn old_list_cell(work: &PolygonWork, n: usize) -> &Dword {
    list_cell(work, work.list.len() + n)
}

/// The first record of the pool the clip makes its records in (`g_polygonNextRecord`
/// starts at it).
pub fn pool_start() -> u32 {
    crate::scene::point_records::of_pool(0)
}

/// The `bytes` from byte `offset` of object kind `kind`'s material record (0x80 bytes
/// each), read with the bound of the polygon tables around the records, not the records
/// alone: `scene::set_object_material` is given kind -1 and reads the record before the
/// first (the texture infos' tail), and a read can run on into the next record, as the
/// original does. A read past the tables is refused.
pub fn material_run(
    tables: &crate::scene::polygon_store::PolygonTables,
    kind: u32,
    offset: u32,
    bytes: &mut [u8],
) -> Result<(), BadInput> {
    use crate::scene::polygon_store::PolygonTables;
    let at = (core::mem::offset_of!(PolygonTables, object_materials) as u32)
        .wrapping_add(kind.wrapping_mul(0x80))
        .wrapping_add(offset);
    crate::cells::load_cells(tables, at as usize, bytes)
}

/// The byte at `offset` of object kind `kind`'s material record (as `material_run`).
pub fn material_run_byte(
    tables: &crate::scene::polygon_store::PolygonTables,
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
    tables: &crate::scene::polygon_store::PolygonTables,
    kind: u32,
    offset: u32,
) -> Result<u16, BadInput> {
    let mut word = [0u8; 2];
    material_run(tables, kind, offset, &mut word)?;
    Ok(u16::from_le_bytes(word))
}

/// The byte `offset` into the polygon tables' field of `len` bytes at `field`; the field's
/// end is the limit, a read on into the next record inside it as the original's arithmetic
/// does.
fn field_byte(
    tables: &crate::scene::polygon_store::PolygonTables,
    field: usize,
    len: usize,
    offset: u32,
    what: &'static str,
) -> Result<u8, BadInput> {
    let offset = offset as usize;
    if offset >= len {
        return Err(BadInput(what));
    }
    let mut byte = [0u8];
    crate::cells::load_cells(tables, field + offset, &mut byte)?;
    Ok(byte[0])
}

/// Byte `offset` of texture info `n` (`g_textureInfos` + 0x40 n): an offset past the
/// record reads on into the next, as the original's arithmetic does.
pub fn texture_info_byte(
    tables: &crate::scene::polygon_store::PolygonTables,
    n: u32,
    offset: u32,
) -> Result<u8, BadInput> {
    use crate::scene::polygon_store::PolygonTables;
    field_byte(
        tables,
        core::mem::offset_of!(PolygonTables, texture_infos),
        core::mem::size_of_val(&tables.texture_infos),
        n.wrapping_mul(0x40).wrapping_add(offset),
        "a texture info's byte past the table",
    )
}

/// Byte `offset` of object kind `kind`'s material record (`g_objectMaterials` + 0x80 kind;
/// kind -1 the record before the first, as the original reads it).
pub fn material_byte(
    tables: &crate::scene::polygon_store::PolygonTables,
    kind: u32,
    offset: u32,
) -> Result<u8, BadInput> {
    use crate::scene::polygon_store::PolygonTables;
    field_byte(
        tables,
        core::mem::offset_of!(PolygonTables, object_materials),
        core::mem::size_of_val(&tables.object_materials),
        kind.wrapping_mul(0x80).wrapping_add(offset),
        "an object's material byte past the table",
    )
}

/// A polygon's entries: on the stack for all but the largest polygons.
pub type Listed<'w> = smallvec::SmallVec<[&'w PolygonEntry; 32]>;

/// The polygon's entries in the list's order. Refuses what the originals go wrong on:
/// more vertices than the vertex buffer holds, a list entry past the table.
pub fn listed<'w>(
    work: &'w PolygonWork,
    poly: &crate::scene::polygon_work::PolygonWork,
) -> Result<Listed<'w>, BadInput> {
    let count: u32 = poly.polygon_vertex_count.get();
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
