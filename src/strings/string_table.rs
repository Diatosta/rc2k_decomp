//! The packed string table and the two string-pointer tables the string hooks read.

use crate::framework::{game_addr, game_ptr, game_read, rva};
use rc2k_core::strings::table::StringTable;
use std::ops::RangeInclusive;

const BLOB_PTR_RVA: usize = rva(0x4E07D4); // g_stringTable: pointer to the packed blob
const PTRS_HIGH_RVA: usize = rva(0x525DE8); // dword_525DE8[id], ids with bit 0x1000
const PTRS_LOW_RVA: usize = rva(0x95D664); // src[id], ids 0x5A5..=0x658

pub const LOW_IDS: RangeInclusive<u32> = rc2k_core::strings::table::POINTER_IDS;

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

/// Where `strings::compile_lexicon` puts the table (`g_lexicon`), and its room.
const LEXICON_RVA: usize = rva(0x4E07F8);
const LEXICON_ROOM: usize = rc2k_core::strings::lexicon::TABLE_ROOM;

/// The table `g_stringTable` points at, as the core reads it: `None` before the language is
/// loaded, or if it points anywhere but the lexicon's room.
pub fn table() -> Option<StringTable<'static>> {
    let b = blob();
    (b == game_addr(LEXICON_RVA) as *const u8)
        .then(|| StringTable::new(unsafe { std::slice::from_raw_parts(b, LEXICON_ROOM) }))
}

/// The number of entries (converted: `rc2k_core::strings::table::StringTable::entry_count`).
pub fn entry_count() -> usize {
    table().map_or(0, |t| t.entry_count())
}

/// True if `id` is an entry of the packed table and is expanded from it (not taken from one
/// of the pointer tables): the ids a test can use without knowing the pointer tables.
pub fn is_table_id(id: u32) -> bool {
    (id as usize) < entry_count()
        && id & 0x1000 == 0
        && !(LOW_IDS.contains(&id) && unsafe { !ptr_low(id).is_null() })
}
