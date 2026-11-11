//! The packed string table and the two string-pointer tables the string functions read.

use crate::errors::BadInput;
use crate::pointers::GamePtr;
use rc2k_core::strings::table::StringTable;

// `g_stringTable` (0x4E07D4), the packed table's address, is not kept: the original sets it
// to the lexicon only (`compile_lexicon`, `load_language`) and the table is always read from
// `PUBLISHED`.
// Still the image's: indexed by the id itself (`base + 4 * id`), they reach other
// variables' pointers (made texts), set by other modules by address.
const PTRS_HIGH: usize = 0x525DE8; // dword_525DE8[id], ids with bit 0x1000
const PTRS_LOW: usize = 0x95D664; // src[id], ids 0x5A5..=0x658

/// Each slot by its own address: slots fall in moved ranges (the driver names' pointers,
/// 0x95ECF8 .., are `src[0x5A5..]`), which the table's base does not follow.
fn table_entry(table: usize, id: u32) -> Result<u32, BadInput> {
    crate::moved::named(table.wrapping_add(4 * id as usize))?.read::<u32>("a string pointer")
}
/// The address of the made text of `id` (0 if none).
pub fn ptr_high(id: u32) -> Result<u32, BadInput> {
    let entry = table_entry(PTRS_HIGH, id)?;
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
    table_entry(PTRS_LOW, id)
}

/// Where `strings::compile_lexicon` puts the table, and its room (`g_lexicon` in IDA, 0x4E07F8 .. the keyboard file at 0x4F07C8).
pub const LEXICON_ROOM: usize = rc2k_core::strings::lexicon::TABLE_ROOM;
pub static LEXICON: crate::session::Bytes<LEXICON_ROOM> =
    crate::session::Bytes::new([0; LEXICON_ROOM]);

/// The lexicon's room as a place bounded by it.
pub fn lexicon_room() -> GamePtr {
    GamePtr::to(&LEXICON)
}

/// The finished table as the core reads it: a copy of the lexicon's room, made once its bytes
/// are written (`publish`). Readers never see the room itself, so a text being looked up cannot
/// meet a language being loaded.
static PUBLISHED: std::sync::RwLock<Option<&'static [u8]>> = std::sync::RwLock::new(None);

/// Makes the room's bytes the table the game reads. Called at most twice, while the language
/// loads (compiled, then corrected), so the copies kept for the program's life stay small.
pub fn publish() -> Result<(), BadInput> {
    let room = lexicon_room();
    let copy = room.read_bytes(LEXICON_ROOM, "the lexicon's room")?;
    *PUBLISHED
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Vec::leak(copy));
    Ok(())
}

/// The table `g_stringTable` points at, as the core reads it: `None` before the language is
/// loaded.
pub fn table() -> Option<StringTable<'static>> {
    PUBLISHED
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .map(StringTable::new)
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
