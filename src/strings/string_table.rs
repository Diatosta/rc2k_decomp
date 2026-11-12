//! The packed string table and the two string-pointer tables the string functions read.

use crate::errors::BadInput;
use rc2k_core::strings::table::StringTable;

// `g_stringTable` (0x4E07D4), the packed table's address, is not kept: the original sets it
// to the lexicon only (`compile_lexicon`, `load_language`) and the table is always read from
// `PUBLISHED`.

/// The ids of the texts the game keeps in its own variables (the original's pointer table
/// indexed by the id): the build notice, the screens' text buffers, the drive rows, the
/// installer's languages and the credits.
pub const MADE_TEXT_IDS: core::ops::Range<u32> = 0x1000..0x1040;

/// Where a made text is: the variable holding it and the byte offset in it. The text runs
/// on to its NUL inside that variable, which may be past the field it starts in.
enum MadeText {
    Notice(usize),
    Buffers(usize),
    DriveRows(usize),
    Names(usize),
    Vars(usize),
}

/// Where the made text of `id` is. An id off `MADE_TEXT_IDS` has no text.
fn made_place(id: u32) -> Result<MadeText, BadInput> {
    use crate::install::store::{DriveRows, InstallVars};
    use crate::install::vars::InstallerNames;
    use crate::menu::build_notice::BuildNotice;
    use crate::screens::text_buffers_store::TextBuffers;
    use core::mem::offset_of;

    /// Byte offsets from `credits_a` of the twelve texts in the installer's variables.
    const CREDITS: [u32; 12] = [
        0x0, 0x2, 0x9, 0xB, 0x1B, 0x3D, 0x1CF, 0x36, 0x5E, 0x1FB, 0x69, 0x20C,
    ];
    if !MADE_TEXT_IDS.contains(&id) {
        return Err(BadInput("a string id's text at no place"));
    }
    let n = (id - MADE_TEXT_IDS.start) as usize;
    Ok(match n {
        0 => MadeText::Notice(0),
        1 => MadeText::Notice(offset_of!(BuildNotice, notice_second)),
        2 => MadeText::Notice(offset_of!(BuildNotice, notice_second) + 0x36),
        3..=6 => MadeText::Buffers(offset_of!(TextBuffers, stage_heading) + 0x100 * (n - 3)),
        7..=16 => MadeText::Buffers(offset_of!(TextBuffers, page_numbers) + 4 * (n - 7)),
        17..=42 => MadeText::DriveRows(offset_of!(DriveRows, drive_rows) + 0x10 * (n - 17)),
        43..=51 => MadeText::Names(offset_of!(InstallerNames, languages) + 0x80 * (n - 43)),
        _ => MadeText::Vars(offset_of!(InstallVars, credits_a) + CREDITS[n - 52] as usize),
    })
}

/// The text of the made text `id`, up to its NUL (left out).
pub fn made_text(
    race: &crate::race::owner::Race,
    menus: &crate::menu::owner::Menus,
    install: &crate::install::owner::Install,
    id: u32,
) -> Result<Vec<u8>, BadInput> {
    match made_place(id)? {
        MadeText::Notice(at) => text_in(&menus.panels.notice, at),
        MadeText::Buffers(at) => text_in(race.text_buffers(), at),
        MadeText::DriveRows(at) => text_in(&install.drive_rows, at),
        MadeText::Names(at) => text_in(&install.names, at),
        MadeText::Vars(at) => text_in(&install.vars, at),
    }
}

/// The bytes of `holder` from `offset` to the first NUL (left out); the original reads on
/// past the field the text starts in, so the whole variable is the limit.
fn text_in<T: crate::cells::Cells>(holder: &T, offset: usize) -> Result<Vec<u8>, BadInput> {
    const CHUNK: usize = 16;
    let mut text = Vec::new();
    let mut at = offset;
    while at < size_of::<T>() {
        let mut chunk = [0u8; CHUNK];
        let len = CHUNK.min(size_of::<T>() - at);
        crate::cells::load_cells(holder, at, &mut chunk[..len])?;
        if let Some(end) = chunk[..len].iter().position(|&b| b == 0) {
            text.extend_from_slice(&chunk[..end]);
            return Ok(text);
        }
        text.extend_from_slice(&chunk[..len]);
        at += len;
    }
    Err(BadInput("a string without its end in its block"))
}

/// The text in the driver names' block at byte `slot` (a driver name's slot value), up to its
/// NUL (left out).
pub fn driver_name_text(
    strings: &crate::strings::owner::Strings,
    slot: u32,
) -> Result<Vec<u8>, BadInput> {
    use crate::settings::read_driver_names::NAMES_SIZE;
    let names = &strings.drivers.names;
    let mut text = Vec::new();
    for at in slot as usize..NAMES_SIZE {
        match names.get(at) {
            0 => return Ok(text),
            b => text.push(b),
        }
    }
    Err(BadInput("a string without its end in its block"))
}

/// The offset in `drivers.names` of the name of driver-name id `id` (`POINTER_IDS`), 0 for
/// none: the pointer slot each id has.
pub fn driver_name_slot(
    strings: &crate::strings::owner::Strings,
    id: u32,
) -> Result<u32, BadInput> {
    let ids = rc2k_core::strings::table::POINTER_IDS;
    if !ids.contains(&id) {
        return Err(BadInput("a driver name's id off the names"));
    }
    let slot = (id - ids.start()) as usize;
    Ok(strings.drivers.names.get_u32(4 * slot))
}

/// Where `strings::compile_lexicon` puts the table, and its room (`g_lexicon` in IDA, 0x4E07F8 .. the keyboard file at 0x4F07C8).
pub const LEXICON_ROOM: usize = rc2k_core::strings::lexicon::TABLE_ROOM;
pub static LEXICON: crate::session::GameThread<crate::session::PlainBytes<LEXICON_ROOM>> =
    crate::session::GameThread::new(crate::session::PlainBytes::new([0; LEXICON_ROOM]));

/// The lexicon's room, all of it.
pub fn lexicon_bytes() -> Result<Vec<u8>, BadInput> {
    LEXICON.get().read_at(0, LEXICON_ROOM, "the lexicon's room")
}

/// `table` written from the start of the lexicon's room.
pub fn write_lexicon(table: &[u8]) -> Result<(), BadInput> {
    LEXICON.get().write_at(0, table, "the lexicon's room")
}

/// The finished table as the core reads it: a copy of the lexicon's room, made once its bytes
/// are written (`publish`). Readers never see the room itself, so a text being looked up cannot
/// meet a language being loaded.
static PUBLISHED: std::sync::RwLock<Option<&'static [u8]>> = std::sync::RwLock::new(None);

/// Makes the room's bytes the table the game reads. Called at most twice, while the language
/// loads (compiled, then corrected), so the copies kept for the program's life stay small.
pub fn publish() -> Result<(), BadInput> {
    let copy = lexicon_bytes()?;
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

    /// Where the original's pointer table sent each made text, ids 0x1000 .. 0x103F: the
    /// holder (N the notice text, B the screens' text buffers, D the drive rows, M the
    /// installer's names, V its variables) and the text's offset in the holder.
    const IMAGE_PLACES: [(char, usize); 64] = [
        ('N', 0x0),
        ('N', 0x4e),
        ('N', 0x84),
        ('B', 0x200),
        ('B', 0x300),
        ('B', 0x400),
        ('B', 0x500),
        ('B', 0x604),
        ('B', 0x608),
        ('B', 0x60c),
        ('B', 0x610),
        ('B', 0x614),
        ('B', 0x618),
        ('B', 0x61c),
        ('B', 0x620),
        ('B', 0x624),
        ('B', 0x628),
        ('D', 0x0),
        ('D', 0x10),
        ('D', 0x20),
        ('D', 0x30),
        ('D', 0x40),
        ('D', 0x50),
        ('D', 0x60),
        ('D', 0x70),
        ('D', 0x80),
        ('D', 0x90),
        ('D', 0xa0),
        ('D', 0xb0),
        ('D', 0xc0),
        ('D', 0xd0),
        ('D', 0xe0),
        ('D', 0xf0),
        ('D', 0x100),
        ('D', 0x110),
        ('D', 0x120),
        ('D', 0x130),
        ('D', 0x140),
        ('D', 0x150),
        ('D', 0x160),
        ('D', 0x170),
        ('D', 0x180),
        ('D', 0x190),
        ('M', 0x38e),
        ('M', 0x40e),
        ('M', 0x48e),
        ('M', 0x50e),
        ('M', 0x58e),
        ('M', 0x60e),
        ('M', 0x68e),
        ('M', 0x70e),
        ('M', 0x78e),
        ('V', 0x13c),
        ('V', 0x13e),
        ('V', 0x145),
        ('V', 0x147),
        ('V', 0x157),
        ('V', 0x179),
        ('V', 0x30b),
        ('V', 0x172),
        ('V', 0x19a),
        ('V', 0x337),
        ('V', 0x1a5),
        ('V', 0x348),
    ];

    /// The holder and offset `made_place` gives the made text `id`, in the table's terms.
    fn place_of(id: u32) -> (char, usize) {
        match made_place(id).unwrap() {
            MadeText::Notice(at) => ('N', at),
            MadeText::Buffers(at) => ('B', at),
            MadeText::DriveRows(at) => ('D', at),
            MadeText::Names(at) => ('M', at),
            MadeText::Vars(at) => ('V', at),
        }
    }

    #[test]
    fn made_texts_are_where_the_images_pointers_named() {
        for (n, &place) in IMAGE_PLACES.iter().enumerate() {
            let id = MADE_TEXT_IDS.start + n as u32;
            assert_eq!(place_of(id), place, "{id:#X}");
        }
        let o = crate::owners::boundary();
        let (race, menus, install) = (o.race, o.menus, o.install);
        assert!(made_text(race, menus, install, 0x1040).is_err());
        assert!(made_text(race, menus, install, 0x1FFF).is_err());
    }

    #[test]
    fn caption_is_id_0x1004() {
        assert!(matches!(
            made_place(0x1004).unwrap(),
            MadeText::Buffers(at)
                if at == core::mem::offset_of!(crate::screens::text_buffers_store::TextBuffers, caption)
        ));
    }
}
