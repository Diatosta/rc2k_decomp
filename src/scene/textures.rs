//! The tables that say which texture each of the scene's texture slots shows: shared by the
//! functions that fill them.
//!
//! A face names a slot; `WANTED` says which texture file's picture the slot should
//! hold and `LOADED` which it holds. An object's kind (its file's number) has a first
//! slot and a first texture.

pub const SLOTS: u32 = 0x400;
pub const KINDS: u32 = 0x400;

/// Typed tables (`KIND_TEXTURES.kind_first_slots`, ...) as slices for the core: each read
/// out, `f` run on the copies, each written back (also after a refusal: the core's writes
/// up to it are kept, as when it worked on the tables in place).
pub fn with_cells<const N: usize, R>(
    cells: [&[crate::session::Dword]; N],
    f: impl FnOnce(&mut [Vec<u32>; N]) -> R,
) -> R {
    let mut copies = cells.map(|t| {
        t.iter()
            .map(super::super::session::Dword::get)
            .collect::<Vec<u32>>()
    });
    let result = f(&mut copies);
    for (table, copy) in cells.iter().zip(&copies) {
        for (cell, &v) in table.iter().zip(copy) {
            cell.set(v);
        }
    }
    result
}
