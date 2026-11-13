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

/// Which made text it is: the variable holding it and the text's number in that variable.
/// The text runs on to its NUL inside the variable, which may be past the field it starts in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MadeText {
    /// The build notice's first text, its second, and the second's second line.
    Notice(NoticeLine),
    /// The stage headings, by number.
    StageHeading(usize),
    /// The page numbers, by number.
    PageNumber(usize),
    /// The drive rows, by number.
    DriveRow(usize),
    /// The installer's languages found, by number.
    Language(usize),
    /// The installer's credits, by number.
    Credit(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NoticeLine {
    First,
    Second,
    SecondTail,
}

/// Where the made text of `id` is. An id off `MADE_TEXT_IDS` has no text.
fn made_place(id: u32) -> Result<MadeText, BadInput> {
    if !MADE_TEXT_IDS.contains(&id) {
        return Err(BadInput("a string id's text at no place"));
    }
    let n = (id - MADE_TEXT_IDS.start) as usize;
    Ok(match n {
        0 => MadeText::Notice(NoticeLine::First),
        1 => MadeText::Notice(NoticeLine::Second),
        2 => MadeText::Notice(NoticeLine::SecondTail),
        3..=6 => MadeText::StageHeading(n - 3),
        7..=16 => MadeText::PageNumber(n - 7),
        17..=42 => MadeText::DriveRow(n - 17),
        43..=51 => MadeText::Language(n - 43),
        _ => MadeText::Credit(n - 52),
    })
}

impl MadeText {
    /// The byte offset of the text in its variable.
    fn offset(self) -> usize {
        use crate::install::store::{DriveRows, InstallVars};
        use crate::install::vars::InstallerNames;
        use crate::menu::build_notice::BuildNotice;
        use crate::screens::text_buffers_store::TextBuffers;
        use core::mem::offset_of;

        /// Byte offsets from `credits_a` of the twelve texts in the installer's variables.
        const CREDITS: [usize; 12] = [
            0x0, 0x2, 0x9, 0xB, 0x1B, 0x3D, 0x1CF, 0x36, 0x5E, 0x1FB, 0x69, 0x20C,
        ];
        match self {
            MadeText::Notice(NoticeLine::First) => 0,
            MadeText::Notice(NoticeLine::Second) => offset_of!(BuildNotice, notice_second),
            MadeText::Notice(NoticeLine::SecondTail) => {
                offset_of!(BuildNotice, notice_second) + 0x36
            }
            MadeText::StageHeading(i) => offset_of!(TextBuffers, stage_heading) + 0x100 * i,
            MadeText::PageNumber(i) => offset_of!(TextBuffers, page_numbers) + 4 * i,
            MadeText::DriveRow(i) => offset_of!(DriveRows, drive_rows) + 0x10 * i,
            MadeText::Language(i) => offset_of!(InstallerNames, languages) + 0x80 * i,
            MadeText::Credit(i) => offset_of!(InstallVars, credits_a) + CREDITS[i],
        }
    }
}

/// The text of the made text `id`, up to its NUL (left out).
pub fn made_text(
    race: &crate::race::owner::Race,
    menus: &crate::menu::owner::Menus,
    install: &crate::install::owner::Install,
    id: u32,
) -> Result<Vec<u8>, BadInput> {
    let place = made_place(id)?;
    let at = place.offset();
    match place {
        MadeText::Notice(_) => text_in(&menus.panels.notice, at),
        MadeText::StageHeading(_) | MadeText::PageNumber(_) => text_in(race.text_buffers(), at),
        MadeText::DriveRow(_) => text_in(&install.drive_rows, at),
        MadeText::Language(_) => text_in(&install.names, at),
        MadeText::Credit(_) => text_in(&install.vars, at),
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
        let place = made_place(id).unwrap();
        let holder = match place {
            MadeText::Notice(_) => 'N',
            MadeText::StageHeading(_) | MadeText::PageNumber(_) => 'B',
            MadeText::DriveRow(_) => 'D',
            MadeText::Language(_) => 'M',
            MadeText::Credit(_) => 'V',
        };
        (holder, place.offset())
    }

    #[test]
    fn made_texts_are_where_the_images_pointers_named() {
        for (n, &place) in IMAGE_PLACES.iter().enumerate() {
            let id = MADE_TEXT_IDS.start + n as u32;
            assert_eq!(place_of(id), place, "{id:#X}");
        }
        let o = crate::owners::Owners::the();
        let (race, menus, install) = (o.race, o.menus, o.install);
        assert!(made_text(race, menus, install, 0x1040).is_err());
        assert!(made_text(race, menus, install, 0x1FFF).is_err());
    }

    #[test]
    fn caption_is_id_0x1004() {
        let place = made_place(0x1004).unwrap();
        assert_eq!(place, MadeText::StageHeading(1));
        assert_eq!(
            place.offset(),
            core::mem::offset_of!(crate::screens::text_buffers_store::TextBuffers, caption)
        );
    }
}
