//! The packed string table and the two string-pointer tables the string hooks read.

use crate::errors::BadInput;
use crate::framework::{game_addr, rva};
use crate::moved::Place;
use crate::pointers::GamePtr;
use rc2k_core::strings::table::StringTable;
use std::sync::atomic::{AtomicU32, Ordering};

/// The packed string table's address (0: none yet): out of the image since 2026-10-24
/// (phase 5; it was `g_stringTable`, 0x4E07D4). Set by `strings::compile_lexicon` and
/// `strings::load_language`, always to `LEXICON`.
pub static STRING_TABLE: AtomicU32 = AtomicU32::new(0);
// Still the image's: indexed by the id itself (`base + 4 * id`), they reach other
// variables' pointers (made texts), set by other modules by address.
const PTRS_HIGH_RVA: usize = rva(0x525DE8); // dword_525DE8[id], ids with bit 0x1000
const PTRS_LOW_RVA: usize = rva(0x95D664); // src[id], ids 0x5A5..=0x658

/// The table's address now.
pub fn blob() -> *const u8 {
    crate::pointers::host(STRING_TABLE.load(Ordering::Relaxed)) as *const u8
}

/// Points the string table at the lexicon (`g_stringTable` = `g_lexicon`).
pub fn use_lexicon() {
    STRING_TABLE.store(crate::pointers::game_address(lexicon()), Ordering::Relaxed);
}

/// Each slot by its own address: slots fall in moved ranges (the driver names' pointers,
/// 0x95ECF8 .., are `src[0x5A5..]`), which the table's base does not follow.
fn table_entry(table_rva: usize, id: u32) -> Result<u32, BadInput> {
    GamePtr::new(game_addr(table_rva.wrapping_add(4 * id as usize))).read::<u32>("a string pointer")
}
/// The address of the made text of `id` (0 if none).
pub fn ptr_high(id: u32) -> Result<u32, BadInput> {
    let entry = table_entry(PTRS_HIGH_RVA, id)?;
    if !(MADE_TEXT_IDS.contains(&id) && entry != 0) {
        return Ok(entry);
    }
    crate::moved::home_of_rva((entry as usize).wrapping_sub(crate::framework::IMAGE_BASE))
        .map(|home| home as u32)
        .ok_or(BadInput("a string id's text at no place"))
}

/// The ids whose slots are `InstallVars::text_pointers` (0x529DE8 is `dword_525DE8[0x1000]`),
/// which keep IDA's addresses as names.
pub const MADE_TEXT_IDS: core::ops::Range<u32> = 0x1000..0x1040;
/// The address of the text of `id` in the pointer table (0 if none).
pub fn ptr_low(id: u32) -> Result<u32, BadInput> {
    table_entry(PTRS_LOW_RVA, id)
}

/// Whether IDA's `ida` is a slot of `MADE_TEXT_IDS` (`moved::point_into_homes`).
pub fn holds_handle(ida: usize) -> bool {
    (0x529DE8..0x529EE8).contains(&ida)
}

/// Where `strings::compile_lexicon` puts the table, and its room: out of the image since
/// 2026-10-24 (phase 5; it was `g_lexicon`, 0x4E07F8 .. the keyboard file at 0x4F07C8).
pub const LEXICON_ROOM: usize = rc2k_core::strings::lexicon::TABLE_ROOM;
pub static LEXICON: Place<[u8; LEXICON_ROOM]> = Place::new([0; LEXICON_ROOM]);

pub fn lexicon() -> *mut u8 {
    LEXICON.ptr().cast()
}

/// The table `g_stringTable` points at, as the core reads it: `None` before the language is
/// loaded, or if it points anywhere but the lexicon's room.
pub fn table() -> Option<StringTable<'static>> {
    let b = blob();
    std::ptr::eq(b, lexicon())
        .then(|| StringTable::new(unsafe { std::slice::from_raw_parts(b, LEXICON_ROOM) }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn made_text_ids_find_their_texts() {
        for id in MADE_TEXT_IDS {
            assert_ne!(ptr_high(id).unwrap(), 0, "{id:#X}");
        }
        assert_eq!(
            ptr_high(0x1004).unwrap(),
            crate::screens::text_buffers_store::TEXT_BUFFERS
                .caption
                .addr()
        );
    }
}
