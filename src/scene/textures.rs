//! The tables that say which texture each of the scene's texture slots shows: shared
//! by the hooks that fill them.
//!
//! A face names a slot; `WANTED` says which texture file's picture the slot should
//! hold and `LOADED` which it holds. An object's kind (its file's number) has a first
//! slot and a first texture.

use crate::framework::{game_addr, game_read, rva};

pub const WANTED_RVA: usize = rva(0x5DA968); // dword_5DA968
pub const LOADED_RVA: usize = rva(0x5DB968); // dword_5DB968
pub const SLOTS: u32 = 0x400;
pub const KIND_TEXTURE_RVA: usize = rva(0x5D0AE8); // dword_5D0AE8
pub const KIND_SLOT_RVA: usize = rva(0x5D1AE8); // dword_5D1AE8
pub const KINDS: u32 = 0x400;
pub const TEXTURE_PLACE_RVA: usize = rva(0x5D8968); // dword_5D8968
pub const PLACE_SLOT_RVA: usize = rva(0x5D3AE8); // dword_5D3AE8
pub const PLACES: u32 = 0xFA0;
pub const SLOT_COUNT_RVA: usize = rva(0x5DC970); // dword_5DC970
pub const LOADS_RVA: usize = rva(0x6415F8); // dword_6415F8

/// One of the game's tables of dwords as a slice, for the core.
///
/// # Safety
/// The table must be `entries` dwords at a 4-aligned RVA of the image, and nothing else
/// may use it while the slice lives (the hooks run with the game paused or on its thread).
pub unsafe fn slice(rva: usize, entries: u32) -> &'static mut [u32] {
    unsafe { std::slice::from_raw_parts_mut(game_addr(rva) as *mut u32, entries as usize) }
}

pub unsafe fn entry(rva: usize, n: u32) -> u32 {
    unsafe { game_read(rva + n as usize * 4) }
}
