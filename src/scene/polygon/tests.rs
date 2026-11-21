use super::*;
use crate::scene::polygon_store::PolygonTables;
use core::mem::size_of_val;

/// The bytes of the textures' records.
const INFOS_LEN: usize = 256 * 0x40;
/// The bytes of the textures' records and the materials after them.
const TABLES_LEN: usize = INFOS_LEN + 1200 * 0x80;

/// The tables with dword `k` (the records' dwords in order) set to `k` times a constant.
fn filled() -> Box<PolygonTables> {
    let t = Box::new(PolygonTables::new());
    let words = t
        .texture_infos
        .iter()
        .flat_map(|info| &info.words)
        .chain(t.object_materials.iter().flat_map(|m| &m.words));
    for (k, word) in words.enumerate() {
        word.set((k as u32).wrapping_mul(0x9E37_79B1));
    }
    t
}

/// The tables' bytes `at ..` read in the files' order: refused past the end.
fn loaded(at: usize, into: &mut [u8]) -> Result<(), BadInput> {
    if at
        .checked_add(into.len())
        .is_none_or(|end| end > TABLES_LEN)
    {
        return Err(BadInput(""));
    }
    for (n, byte) in into.iter_mut().enumerate() {
        let a = at + n;
        *byte = ((a / 4) as u32).wrapping_mul(0x9E37_79B1).to_le_bytes()[a % 4];
    }
    Ok(())
}

/// The byte readers as they read through the whole tables' bytes.
fn old_field_byte(field: usize, len: usize, offset: u32) -> Result<u8, BadInput> {
    let offset = offset as usize;
    if offset >= len {
        return Err(BadInput(""));
    }
    let mut byte = [0u8];
    loaded(field + offset, &mut byte)?;
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
            old_field_byte(0, infos, offset).ok(),
            "info byte {offset}"
        );
    }
    let materials = size_of_val(&t.object_materials);
    for offset in (0..materials as u32 + 8).step_by(3) {
        assert_eq!(
            field_byte(&t.object_materials, offset, "x").ok(),
            old_field_byte(INFOS_LEN, materials, offset).ok(),
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
    let base = INFOS_LEN as u32;
    for kind in [u32::MAX, 0, 1, 1198, 1199, 1200] {
        for offset in [0u32, 1, 3, 0x1F, 0x7C, 0x7F, 0x80, 0x100] {
            for len in [1usize, 2, 4, 8] {
                let at = base
                    .wrapping_add(kind.wrapping_mul(0x80))
                    .wrapping_add(offset);
                let (mut old, mut new) = ([0u8; 8], [0u8; 8]);
                let o = loaded(at as usize, &mut old[..len]);
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
