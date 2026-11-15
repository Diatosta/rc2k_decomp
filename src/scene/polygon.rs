//! The polygon being worked on, as the functions of `scene` share it: entries of the work table
//! picked through the list (`draw_polygon.rs` has an entry's layout).

use crate::errors::BadInput;
pub use crate::scene::draw_polygon::{ENTRIES, ENTRY_LEN, MOST};
use crate::scene::polygon_store::{PolygonEntry, PolygonWork};
use crate::session::PlainDword;

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
        .or_else(|| work.unknown_2400.get(after_old))
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
    tables: &crate::scene::polygon_store::PolygonTables,
    kind: u32,
    offset: u32,
    bytes: &mut [u8],
) -> Result<(), BadInput> {
    // The materials follow the texture infos, so kind -1 is the infos' last record.
    let at = (size_of_val(&tables.texture_infos) as u32)
        .wrapping_add(kind.wrapping_mul(0x80))
        .wrapping_add(offset);
    crate::session::read_run_bytes(
        &[&tables.texture_infos, &tables.object_materials],
        at as usize,
        bytes,
        "a material record's bytes",
    )
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

/// The byte `offset` into the run `field` (one of the polygon tables' arrays of records); the
/// field's end is the limit, a read on into the next record inside it.
fn field_byte(
    field: &dyn crate::session::Run,
    offset: u32,
    what: &'static str,
) -> Result<u8, BadInput> {
    let mut byte = [0u8];
    crate::session::read_run_bytes(&[field], offset as usize, &mut byte, what)
        .map_err(|_| BadInput(what))?;
    Ok(byte[0])
}

/// Byte `offset` of texture info `n`: an offset past the record reads on into the next.
pub fn texture_info_byte(
    tables: &crate::scene::polygon_store::PolygonTables,
    n: u32,
    offset: u32,
) -> Result<u8, BadInput> {
    field_byte(
        &tables.texture_infos,
        n.wrapping_mul(0x40).wrapping_add(offset),
        "a texture info's byte past the table",
    )
}

/// A polygon's entries: on the stack for all but the largest polygons.
pub type Listed<'w> = smallvec::SmallVec<[&'w PolygonEntry; 32]>;

/// The polygon's entries in the list's order. Refuses more vertices than the vertex buffer
/// holds and a list entry past the table.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::polygon_store::PolygonTables;
    use core::mem::{offset_of, size_of};

    fn filled() -> Box<PolygonTables> {
        let t: Box<PolygonTables> = Box::new(crate::cells::zeroed_cells());
        for k in 0..size_of::<PolygonTables>() / 4 {
            crate::cells::store_value::<u32>(&*t, 4 * k, (k as u32).wrapping_mul(0x9E37_79B1))
                .unwrap();
        }
        t
    }

    /// The byte readers as they read through the whole tables' bytes.
    fn old_field_byte(
        t: &PolygonTables,
        field: usize,
        len: usize,
        offset: u32,
    ) -> Result<u8, BadInput> {
        let offset = offset as usize;
        if offset >= len {
            return Err(BadInput(""));
        }
        let mut byte = [0u8];
        crate::cells::load_cells(t, field + offset, &mut byte)?;
        Ok(byte[0])
    }

    /// Every offset up to the field's end (and past it) reads as the old byte read; an
    /// offset past the end is refused by both.
    #[test]
    fn field_bytes_read_as_the_byte_read() {
        let t = filled();
        let infos = size_of_val(&t.texture_infos);
        for offset in 0..infos as u32 + 8 {
            assert_eq!(
                field_byte(&t.texture_infos, offset, "x").ok(),
                old_field_byte(&t, 0, infos, offset).ok(),
                "info byte {offset}"
            );
        }
        let materials = size_of_val(&t.object_materials);
        for offset in (0..materials as u32 + 8).step_by(3) {
            assert_eq!(
                field_byte(&t.object_materials, offset, "x").ok(),
                old_field_byte(
                    &t,
                    offset_of!(PolygonTables, object_materials),
                    materials,
                    offset
                )
                .ok(),
                "material byte {offset}"
            );
        }
        assert!(texture_info_byte(&t, 255, 0x40).is_err());
    }

    /// A material record's run reads as the old read over the whole tables, kind -1 (the
    /// infos' last record) and the run-on past the last record included.
    #[test]
    fn material_runs_read_as_the_byte_read() {
        let t = filled();
        let base = offset_of!(PolygonTables, object_materials) as u32;
        for kind in [u32::MAX, 0, 1, 1198, 1199, 1200] {
            for offset in [0u32, 1, 3, 0x1F, 0x7C, 0x7F, 0x80, 0x100] {
                for len in [1usize, 2, 4, 8] {
                    let at = base
                        .wrapping_add(kind.wrapping_mul(0x80))
                        .wrapping_add(offset);
                    let (mut old, mut new) = ([0u8; 8], [0u8; 8]);
                    let o = crate::cells::load_cells(&*t, at as usize, &mut old[..len]);
                    let n = material_run(&t, kind, offset, &mut new[..len]);
                    assert_eq!(
                        o.is_ok(),
                        n.is_ok(),
                        "kind {kind} offset {offset} len {len}"
                    );
                    if o.is_ok() {
                        assert_eq!(old, new);
                    }
                }
            }
        }
    }
}
