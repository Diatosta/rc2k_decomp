use super::*;
use crate::scene::polygon_store::PolygonTables;
use core::mem::{offset_of, size_of};

fn filled() -> Box<PolygonTables> {
    let t: Box<PolygonTables> = Box::new(crate::memory::cells::zeroed_cells());
    for k in 0..size_of::<PolygonTables>() / 4 {
        crate::memory::cells::store_value::<u32>(&*t, 4 * k, (k as u32).wrapping_mul(0x9E37_79B1))
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
    crate::memory::cells::load_cells(t, field + offset, &mut byte)?;
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
                let o = crate::memory::cells::load_cells(&*t, at as usize, &mut old[..len]);
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
