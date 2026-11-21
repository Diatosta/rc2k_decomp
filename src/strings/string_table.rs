//! The packed string table and the two string-pointer tables the string functions read.

use crate::errors::BadInput;
use crate::memory::place::Run;
use rc2k_core::strings::table::StringTable;

// The string table pointer is not kept: the table is always read from `Lexicon::published`.

/// The ids of the texts the game keeps in its own variables (the pointer table indexed by the id):
/// The build notice, the screens' text buffers, the drive rows, the installer's languages and the
/// credits.
const MADE_TEXT_IDS: core::ops::Range<u32> = 0x1000..0x1040;

/// Which made text it is: the variable holding it and the text's number in that variable. The text
/// runs on to its NUL inside the variable, which may be past the field it starts in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MadeText {
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
pub enum NoticeLine {
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
    /// The text's byte in its variable, counted from the first of the buffers in `chain`.
    fn offset(self, chain: &[&dyn Run]) -> usize {
        /// Byte offsets from `credits_a` of the twelve texts in the installer's variables.
        const CREDITS: [usize; 12] = [
            0x0, 0x2, 0x9, 0xB, 0x1B, 0x3D, 0x1CF, 0x36, 0x5E, 0x1FB, 0x69, 0x20C,
        ];
        // The bytes before the chain's buffer `n`.
        let before = |n: usize| chain[..n].iter().map(|buffer| buffer.len()).sum::<usize>();
        match self {
            MadeText::Notice(NoticeLine::First) => 0,
            MadeText::Notice(NoticeLine::Second) => before(1),
            MadeText::Notice(NoticeLine::SecondTail) => before(1) + 0x36,
            MadeText::StageHeading(i) => before(1) * i,
            MadeText::PageNumber(i) => before(5) + 4 * i,
            MadeText::DriveRow(i) => 0x10 * i,
            MadeText::Language(i) => 0x80 * i,
            MadeText::Credit(i) => CREDITS[i],
        }
    }

    /// The variable's buffers from the one the made texts are counted from, in the order they
    /// follow one another: a text runs on past its field into the next. Each text is the
    /// buffers' of one of the two owners; `None` when neither holds it.
    fn chain<'a>(
        self,
        menus: &'a dyn MadeTextBuffers,
        install: &'a dyn MadeTextBuffers,
    ) -> Option<Vec<&'a dyn Run>> {
        menus.chain(self).or_else(|| install.chain(self))
    }
}

/// An owner of the variables some made texts live in (the menus' notice and text buffers, the
/// installer's rows, names and credits): the front end implements it, so that building the
/// texts does not depend on it.
pub trait MadeTextBuffers {
    /// The buffers of `text` from the one it is counted from, in memory order; `None` when
    /// `text` is not in this owner's variables.
    fn chain(&self, text: MadeText) -> Option<Vec<&dyn Run>>;
}

/// The text of the made text `id`, up to its NUL (left out).
pub fn made_text(
    menus: &dyn MadeTextBuffers,
    install: &dyn MadeTextBuffers,
    id: u32,
) -> Result<Vec<u8>, BadInput> {
    let place = made_place(id)?;
    let chain = place
        .chain(menus, install)
        .ok_or(BadInput("a made text in no owner's variables"))?;
    text_across(&chain, place.offset(&chain))
}

/// The bytes from `offset` into the first of `buffers` to the first NUL (left out); the text runs
/// on past the field it starts in, so the buffers that follow one another are the limit.
fn text_across(buffers: &[&dyn Run], offset: usize) -> Result<Vec<u8>, BadInput> {
    let mut text = Vec::new();
    let mut skip = offset;
    for buffer in buffers {
        let len = buffer.len();
        if skip >= len {
            skip -= len;
            continue;
        }
        for at in skip..len {
            match buffer.get(at) {
                0 => return Ok(text),
                byte => text.push(byte),
            }
        }
        skip = 0;
    }
    Err(BadInput("a string without its end in its block"))
}

/// The text in the driver names' block at byte `slot` (a driver name's slot value), up to its NUL
/// (left out).
pub(super) fn driver_name_text(
    strings: &crate::strings::owner::Strings,
    slot: u32,
) -> Result<Vec<u8>, BadInput> {
    use crate::strings::owner::NAMES_SIZE;
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

/// The offset in `drivers.names` of the name of driver-name id `id` (`POINTER_IDS`), 0 for none:
/// The pointer slot each id has.
pub(super) fn driver_name_slot(
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

/// The lexicon's room: where `language::compile_lexicon` puts the table.
pub(super) const LEXICON_ROOM: usize = rc2k_core::strings::lexicon::TABLE_ROOM;

/// The lexicon's room, all of it.
pub fn lexicon_bytes(strings: &crate::strings::owner::Strings) -> Result<Vec<u8>, BadInput> {
    strings
        .lexicon
        .room
        .read_at(0, LEXICON_ROOM, "the lexicon's room")
}

/// `table` written from the start of the lexicon's room.
pub fn write_lexicon(
    strings: &crate::strings::owner::Strings,
    table: &[u8],
) -> Result<(), BadInput> {
    strings
        .lexicon
        .room
        .write_at(0, table, "the lexicon's room")
}

/// Makes the room's bytes the table the game reads. Called at most twice, while the language loads
/// (compiled, then corrected), so the copies kept for the program's life stay small.
pub fn publish(strings: &crate::strings::owner::Strings) -> Result<(), BadInput> {
    let copy = lexicon_bytes(strings)?;
    *strings
        .lexicon
        .published
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Vec::leak(copy));
    Ok(())
}

/// The table the game reads, as the core reads it: `None` before the language is loaded.
pub fn table(strings: &crate::strings::owner::Strings) -> Option<StringTable<'static>> {
    strings
        .lexicon
        .published
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .map(StringTable::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// How many ids `the_installers_made_texts_read_as_before` resolves, and their hash.
    const INSTALL_TEXTS_COUNT: usize = 47;
    const INSTALL_TEXTS_HASH: u64 = 0xa852_4ba6_54d7_42e9;

    /// Where each made text, ids 0x1000 .. 0x103F, lives: the holder (N the notice text, B the
    /// screens' text buffers, D the drive rows, M the installer's names, V its variables) and the
    /// text's offset in the holder.
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

    /// The holder and offset `made_place` gives the made text `id`, in the table's terms (the
    /// offset in the variable: the chain counts from the buffer `first_byte` bytes into it).
    fn place_of(id: u32) -> (char, usize) {
        let o = crate::test_support::own::owners();
        let place = made_place(id).unwrap();
        let (holder, first_byte) = match place {
            MadeText::Notice(_) => ('N', 0),
            MadeText::StageHeading(_) | MadeText::PageNumber(_) => ('B', 0x200),
            MadeText::DriveRow(_) => ('D', 0),
            MadeText::Language(_) => ('M', 0x38e),
            MadeText::Credit(_) => ('V', 0x13c),
        };
        let chain = place.chain(o.menus, o.install).unwrap();
        (holder, first_byte + place.offset(&chain))
    }

    #[test]
    fn made_texts_are_where_the_images_pointers_named() {
        for (n, &place) in IMAGE_PLACES.iter().enumerate() {
            let id = MADE_TEXT_IDS.start + n as u32;
            assert_eq!(place_of(id), place, "{id:#X}");
        }
        let o = crate::test_support::own::owners();
        let (menus, install) = (o.menus, o.install);
        assert!(made_text(menus, install, 0x1040).is_err());
        assert!(made_text(menus, install, 0x1FFF).is_err());
    }

    #[test]
    fn caption_is_id_0x1004() {
        let o = crate::test_support::own::owners();
        let place = made_place(0x1004).unwrap();
        assert_eq!(place, MadeText::StageHeading(1));
        let chain = place.chain(o.menus, o.install).unwrap();
        assert_eq!(place.offset(&chain), chain[0].len());
        assert!(std::ptr::addr_eq(
            chain[1],
            &raw const o.menus.text_buffers.caption
        ));
    }

    /// Every made text of the installer's stores (the drive rows, the languages, the credits),
    /// resolved on the stores' starting values: a hash of the ids' texts, held against the
    /// build before the installer's pages were typed (`RC2K_PRINT=1` prints it).
    #[test]
    fn the_installers_made_texts_read_as_before() {
        let _lock = crate::test_support::test_lock();
        let o = crate::test_support::own::owners();
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        let mut lines = Vec::new();
        for id in MADE_TEXT_IDS.start + 17..MADE_TEXT_IDS.end {
            let line = match made_text(o.menus, o.install, id) {
                Ok(text) => format!("{id:#x}: {text:?}"),
                Err(e) => format!("{id:#x}: refused {e:?}"),
            };
            for b in line.bytes().chain(std::iter::once(b'\n')) {
                hash = (hash ^ u64::from(b)).wrapping_mul(0x100_0000_01b3);
            }
            lines.push(line);
        }
        if std::env::var_os("RC2K_PRINT").is_some() {
            eprintln!("INSTALL_TEXTS_HASH {hash:016x} over {} ids", lines.len());
        }
        assert_eq!(
            (lines.len(), hash),
            (INSTALL_TEXTS_COUNT, INSTALL_TEXTS_HASH),
            "{lines:?}"
        );
    }
}
