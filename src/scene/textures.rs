//! The tables that say which texture each of the scene's texture slots shows: shared
//! by the hooks that fill them.
//!
//! A face names a slot; `WANTED` says which texture file's picture the slot should
//! hold and `LOADED` which it holds. An object's kind (its file's number) has a first
//! slot and a first texture.

use crate::framework::{game_addr, game_read, rva};
use crate::service::harness::Region;

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

pub fn table(rva: usize, entries: u32) -> Region {
    (game_addr(rva) as u32, entries as usize * 4)
}

pub unsafe fn entry(rva: usize, n: u32) -> u32 {
    unsafe { game_read(rva + n as usize * 4) }
}

/// Numbers for a test's table, by lot: about one in `gap` below `below`, the others -1.
pub fn test_table(seed: &mut u32, entries: u32, gap: u32, below: u32) -> Vec<u32> {
    (0..entries)
        .map(|_| {
            *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let lot = *seed >> 8;
            if lot.is_multiple_of(gap) {
                (lot / gap) % below
            } else {
                u32::MAX
            }
        })
        .collect()
}

pub unsafe fn write_table(rva: usize, values: &[u32]) {
    unsafe {
        std::ptr::copy_nonoverlapping(
            values.as_ptr() as *const u8,
            game_addr(rva) as *mut u8,
            values.len() * 4,
        );
    }
}
