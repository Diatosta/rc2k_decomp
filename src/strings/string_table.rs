//! The packed string table and the two string-pointer tables the string hooks read.

use crate::framework::{game_ptr, game_read, rva};
use std::ops::RangeInclusive;

const BLOB_PTR_RVA: usize = rva(0x4E07D4); // g_stringTable: pointer to the packed blob
const PTRS_HIGH_RVA: usize = rva(0x525DE8); // dword_525DE8[id], ids with bit 0x1000
const PTRS_LOW_RVA: usize = rva(0x95D664); // src[id], ids 0x5A5..=0x658

pub const LOW_IDS: RangeInclusive<u32> = 0x5A5..=0x658;

/// The game fills this pointer in at runtime, so read it on every use.
pub fn blob() -> *const u8 {
    unsafe { game_read::<u32>(BLOB_PTR_RVA) as usize as *const u8 }
}

unsafe fn table_entry(table_rva: usize, id: u32) -> *const u8 {
    unsafe {
        game_ptr::<u32>(table_rva)
            .wrapping_add(id as usize)
            .read_volatile() as usize as *const u8
    }
}
pub unsafe fn ptr_high(id: u32) -> *const u8 {
    unsafe { table_entry(PTRS_HIGH_RVA, id) }
}
pub unsafe fn ptr_low(id: u32) -> *const u8 {
    unsafe { table_entry(PTRS_LOW_RVA, id) }
}

/// Assumes the 4-byte entry index sits at the start of the blob, so entry 0's offset
/// (where the data begins) / 4 is the entry count. Verify this in IDA.
pub fn entry_count() -> usize {
    let b = blob();
    if b.is_null() {
        return 0;
    }
    let first = unsafe { b.cast::<u32>().read_unaligned() } & 0x00FF_FFFF;
    ((first / 4) as usize).min(0x10000)
}
