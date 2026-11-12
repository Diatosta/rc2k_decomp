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
fn text_in<T: crate::pointers::Cells>(holder: &T, offset: usize) -> Result<Vec<u8>, BadInput> {
    const CHUNK: usize = 16;
    let mut text = Vec::new();
    let mut at = offset;
    while at < size_of::<T>() {
        let mut chunk = [0u8; CHUNK];
        let len = CHUNK.min(size_of::<T>() - at);
        crate::pointers::load_cells(holder, at, &mut chunk[..len])?;
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
pub static LEXICON: crate::session::Bytes<LEXICON_ROOM> =
    crate::session::Bytes::new([0; LEXICON_ROOM]);

/// The lexicon's room, all of it.
pub fn lexicon_bytes() -> Result<Vec<u8>, BadInput> {
    LEXICON.read_at(0, LEXICON_ROOM, "the lexicon's room")
}

/// `table` written from the start of the lexicon's room.
pub fn write_lexicon(table: &[u8]) -> Result<(), BadInput> {
    LEXICON.write_at(0, table, "the lexicon's room")
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

    /// IDA's addresses of the made texts, ids 0x1000 .. 0x103F: where the original's pointer
    /// table sent each.
    const IMAGE_ADDRESSES: [usize; 64] = [
        0x52A263, 0x52A2B1, 0x52A2E7, 0xA8BD54, 0xA8BE54, 0xA8BF54, 0xA8C054, 0xA8C158, 0xA8C15C,
        0xA8C160, 0xA8C164, 0xA8C168, 0xA8C16C, 0xA8C170, 0xA8C174, 0xA8C178, 0xA8C17C, 0x52A30F,
        0x52A31F, 0x52A32F, 0x52A33F, 0x52A34F, 0x52A35F, 0x52A36F, 0x52A37F, 0x52A38F, 0x52A39F,
        0x52A3AF, 0x52A3BF, 0x52A3CF, 0x52A3DF, 0x52A3EF, 0x52A3FF, 0x52A40F, 0x52A41F, 0x52A42F,
        0x52A43F, 0x52A44F, 0x52A45F, 0x52A46F, 0x52A47F, 0x52A48F, 0x52A49F, 0x51BEEE, 0x51BF6E,
        0x51BFEE, 0x51C06E, 0x51C0EE, 0x51C16E, 0x51C1EE, 0x51C26E, 0x51C2EE, 0x529EE8, 0x529EEA,
        0x529EF1, 0x529EF3, 0x529F03, 0x529F25, 0x52A0B7, 0x529F1E, 0x529F46, 0x52A0E3, 0x529F51,
        0x52A0F4,
    ];

    /// IDA's address of the made text `id`: its holder's first byte in the image (the notice
    /// text, the screens' text buffers, the drive rows, the installer's names and its
    /// variables, the last starting a byte before its range) and the text's offset in it.
    fn image_address(id: u32) -> usize {
        match made_place(id).unwrap() {
            MadeText::Notice(at) => 0x52A263 + at,
            MadeText::Buffers(at) => crate::screens::text_buffers_store::START + at,
            MadeText::DriveRows(at) => crate::install::store::DriveRows::START + at,
            MadeText::Names(at) => crate::install::vars::InstallerNames::START + at,
            MadeText::Vars(at) => crate::install::store::InstallVars::START - 1 + at,
        }
    }

    #[test]
    fn made_texts_are_where_the_images_pointers_named() {
        for (n, &ida) in IMAGE_ADDRESSES.iter().enumerate() {
            let id = MADE_TEXT_IDS.start + n as u32;
            assert_eq!(image_address(id), ida, "{id:#X}");
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
