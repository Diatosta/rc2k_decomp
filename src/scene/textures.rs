//! The tables that say which texture each of the scene's texture slots shows: shared by the
//! functions that fill them.
//!
//! A face names a slot; `WANTED` says which texture file's picture the slot should
//! hold and `LOADED` which it holds. An object's kind (its file's number) has a first
//! slot and a first texture.

pub const SLOTS: u32 = 0x400;
pub const KINDS: u32 = 0x400;

/// A table of the typed variables (`KIND_TEXTURES.kind_first_slots`, ...) as a slice, for the core.
///
/// # Safety
/// Nothing else may use the table while the slice lives (the game runs on its thread).
#[allow(clippy::mut_from_ref)] // the statics are the game's one thread's; same as `slice`
pub unsafe fn cells_mut(cells: &'static [crate::session::Dword]) -> &'static mut [u32] {
    unsafe { std::slice::from_raw_parts_mut(cells.as_ptr() as *mut u32, cells.len()) }
}
