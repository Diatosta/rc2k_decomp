//! `strings::load_language` (0x40331F): makes the language ready: its strings, its units,
//! its keyboard and its letters' cases.
//!
//! In:  nothing
//! Out: every register preserved
//! In this order:
//! - `strings::compile_lexicon`; `g_stringTable` is the lexicon; the keyboard layout
//!   (`dword_4F07D4`) is the built-in one at 0x4F7FDC, with `byte_4F07D1` and `byte_4F07D2` 1;
//! - the first `files\%locale\*.loc`, 0x30 bytes to 0x4E0798: none is fatal error
//!   0x80E38381, one that does not start "LOCL" 0x80E38382. Its byte +0xF is the units
//!   (`g_isImperialUnits` and each car's +0x178); its byte +8 (`g_noticeKind`) is read
//!   further down;
//! - the first `files\%locale\*.kim`, up to 0x400 bytes to 0x4F07C8, if there is one: not
//!   starting "KEYB" is fatal error 0x80E383A2. The ten offsets at its +0x30 become
//!   addresses, the layout is the block of 0x14 bytes its byte +0xB chooses, and if its
//!   byte +8 is 0x0C three entries of that layout's fourth table are set (0x1034, 0x1035,
//!   0x1036 at +0x34, +0x36, +0x52);
//! - `sub_4034C2`: the case tables. `g_lowerCase` (0x4F0BC8): every character itself, A .. Z
//!   in lower case, then string 2 as pairs "lower, upper". `byte_4F0CC8`: a copy of that in
//!   which string 3's accented letters become the plain letter before them ("aàáâ.." );
//! - `sub_403462`: with keyboard type 2 (`dword_518404`) a built-in layout takes the file's
//!   place;
//! - `sub_403495`: on a Japanese system (`GetSystemDefaultLCID`, language 0x11) a notice
//!   kind other than 7 is fatal error 0x90006044; with kind 7 `dword_51B980` is 0x2EE;
//! - `sub_416389`: four corrections in the lexicon itself: in string 0x6CA the word
//!   "Bowland" loses its `w`, in 0x6AB "Laukennen" becomes "Laukkanen", and in 0x23B the
//!   word "A7" or "(A7)" is replaced by string 0x23A's "A8" or "(A8)";
//! - `dword_51B9E4` = 1 if the notice kind is 2, string 0x57C is not empty and the file
//!   `;4ereg\eregip.exe` is there, else 0.
//!
//! Callers: `start` and three in the installer.
//! Refuses a string 2 of odd length (the original reads on past it) and one of the four
//! corrections meeting its word where the string is that word alone.
//! Shadow mode does it all twice (Rust, everything put back, the original) and compares.
//! Manual patch with a private copy (0x143 bytes; nine calls and the three jumps to the
//! fatal stubs relocated): a call starts in the first bytes.
//! Self-test: a redirected copy whose `compile_lexicon` only notes itself, run on a fresh
//! Rust-compiled lexicon with everything else it writes blotted out first.

use crate::errors::{BadInput, FatalError, Refused};
use crate::files::read_first_match::read_first_match;
use crate::gfx::owner::gfx;
use crate::input::layouts::layouts;
use crate::menu::owner::menus;
use crate::pointers::GamePtr;
use crate::session::Bytes;
use crate::strings::append_string_by_id::push_string;
use crate::strings::owner::strings;
use crate::strings::same_text::set_case_table;
use crate::strings::string_table::LEXICON_ROOM;
use crate::system::owner::system;
use rc2k_core::strings::case::CaseTable;
use rc2k_core::strings::language::Locale;

// ---- game data ----

/// `aFilesLocaleLoc` (0x4E075C), `aFilesLocaleKim` (0x4E0784).
const LOCALE_FILES: &[u8] = b"files\\%locale\\*.loc";
const KEYBOARD_FILES: &[u8] = b"files\\%locale\\*.kim";
/// The `.loc` file's 0x30 bytes: +0 "LOCL", +8 the notice kind (`g_noticeKind`), +0xF
/// units. Out of the image since 2026-10-24 (phase 5; it was `g_localeFile`, 0x4E0798),
/// with the image's first bytes (`tests::first_values_are_the_games`).
pub const LOCALE_LEN: usize = 0x30;
pub const LOCALE_INIT: [u8; LOCALE_LEN] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x01, 0x01, 0x2F, 0x2E, 0x2C, 0x00,
    0x64, 0x65, 0x66, 0x61, 0x75, 0x6C, 0x74, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// `g_noticeKind` (the `.loc` file's byte +8, 0x4E07A0 in the image): which notice the
/// language shows (2: the builds' notice; 7: Japan).
pub fn notice_kind() -> u8 {
    strings().language.locale.get(8)
}

/// `g_plainLetters` (0x4F0CC8 in the image until 2026-10-24): `g_lowerCase` with the
/// accented letters made plain; made here, read by nothing now. The image's first values.
pub const PLAIN_LETTERS_INIT: [u8; 256] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F,
    0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F,
    0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x3B, 0x3C, 0x3D, 0x3E, 0x3F,
    0x40, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F,
    0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x5B, 0x5C, 0x5D, 0x5E, 0x5F,
    0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F,
    0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x7B, 0x7C, 0x7D, 0x7E, 0x7F,
    0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8E, 0x8F,
    0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0x9B, 0x9C, 0x9D, 0x9E, 0x9F,
    0xA0, 0xA1, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7, 0xA8, 0xA9, 0xAA, 0xAB, 0xAC, 0xAD, 0xAE, 0xAF,
    0xB0, 0xB1, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xBB, 0xBC, 0xBD, 0xBE, 0xBF,
    0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x63, 0x65, 0x65, 0x65, 0x65, 0x69, 0x69, 0x69, 0x69,
    0x64, 0x6E, 0x6F, 0x6F, 0x6F, 0x6F, 0x6F, 0xD7, 0x6F, 0x75, 0x75, 0x75, 0x75, 0x79, 0x74, 0x73,
    0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x63, 0x65, 0x65, 0x65, 0x65, 0x69, 0x69, 0x69, 0x69,
    0x64, 0x6E, 0x6F, 0x6F, 0x6F, 0x6F, 0x6F, 0xF7, 0x6F, 0x75, 0x75, 0x75, 0x75, 0x79, 0x74, 0x79,
];
/// The `.kim` file (`g_keyboardFile`, dword_4F07C8), 0x400 bytes: +0 "KEYB", +8 a kind, +0xA
/// the layout mode, +0xB the layout's number, +0x30 the layouts
/// (`rc2k_core::input::layouts`).
const KEYBOARD_LEN: usize = rc2k_core::input::layouts::FILE_LEN;
/// Where the file is read to (the layouts own the result).
static KEYBOARD_BUFFER: Bytes<KEYBOARD_LEN> = Bytes::new([0; KEYBOARD_LEN]);

// ---- Rust version ----

fn fatal(code: u32, message: &'static str) -> Refused {
    FatalError { code, message }.into()
}

/// String `id` as the original builds it: in the text buffer. Returns its bytes.
fn built(id: u32, with_nul: bool) -> Result<Vec<u8>, BadInput> {
    let mut bytes = Vec::new();
    push_string(&mut bytes, id)?;
    if bytes.len() >= 0x100 {
        return Err(BadInput(
            "a string of the language's set-up longer than expected",
        ));
    }
    let at = GamePtr::to(&strings().text.buffer.text);
    at.write_bytes(&bytes, "the text buffer")?;
    if with_nul {
        at.offset(bytes.len() as u32)
            .write::<u8>(0, "the text buffer")?;
    }
    Ok(bytes)
}

/// `sub_4034C2` (converted: `rc2k_core::strings::case::CaseTable::{for_language, plain_letters}`).
fn make_case_tables() -> Result<(), BadInput> {
    let lower = CaseTable::for_language(&built(2, true)?)?;
    set_case_table(&lower.lower);
    let plain = lower.plain_letters(&built(3, true)?);
    strings().language.plain_letters.write(&plain.lower);
    Ok(())
}

/// `sub_416389` (converted: `rc2k_core::strings::lexicon::correct`), on the lexicon's room.
fn correct_lexicon() -> Result<(), BadInput> {
    let room = crate::strings::string_table::lexicon_room();
    let mut table = room.read_bytes(LEXICON_ROOM, "the lexicon's room")?;
    rc2k_core::strings::lexicon::correct(&mut table)?;
    room.write_bytes(&table, "the lexicon's room")?;
    crate::strings::string_table::publish()
}

/// `compile` stands for `strings::compile_lexicon`.
pub fn load_language(compile: &mut dyn FnMut() -> Result<(), Refused>) -> Result<(), Refused> {
    let strings = strings();
    compile()?;
    layouts().use_built_in();

    let mut locale = strings.language.locale.read();
    let found = read_first_match(LOCALE_FILES, &mut locale, 0)?;
    strings.language.locale.write(&locale);
    if found.is_none() {
        return Err(fatal(0x80E3_8381, ""));
    }
    // Converted: `rc2k_core::strings::language::Locale`.
    let Ok(file) = Locale::parse(&strings.language.locale.read()) else {
        return Err(fatal(0x80E3_8382, ""));
    };
    let units = file.units as u32;
    crate::strings::units::set_units(units);
    // Each car's copy of the units, at its +0x178.
    for player in &menus().play.players.players {
        player.update(|p| p.metric_units = units);
    }

    // Read over the block the last load left (a shorter file leaves its tail).
    KEYBOARD_BUFFER.write(&layouts().file_bytes());
    let mut keyboard = KEYBOARD_BUFFER.read();
    let found = read_first_match(KEYBOARD_FILES, &mut keyboard, 0)?;
    KEYBOARD_BUFFER.write(&keyboard);
    if found.is_some() {
        let keyboard = KEYBOARD_BUFFER.read();
        if &keyboard[..4] != b"KEYB" {
            return Err(fatal(0x80E3_83A2, ""));
        }
        // Converted: `rc2k_core::input::layouts` (the offsets made places, the kind 0x0C
        // patch).
        layouts().use_file(rc2k_core::input::layouts::parse(&keyboard)?);
    }

    make_case_tables()?;

    // `sub_403462`: the NEC keyboard's head (its layout mode) and layout.
    if system().session.machine.keyboard_type.get() == 2 {
        layouts().use_nec();
        crate::install::owner::install()
            .run
            .first_drive
            .set(0x61u32);
    }

    // `sub_403495`
    let kind = file.notice_kind;
    if crate::os::language_id() & 0x3FF == 0x11 && kind != 7 {
        return Err(fatal(
            0x9000_6044,
            "You cannot run this version of RC in Japan\t",
        ));
    }
    if kind == 7 {
        gfx().fonts.glyph_tables.units.set(0x2EEu32);
    }

    correct_lexicon()?;

    crate::install::owner::install()
        .run
        .offer_registration
        .set(0u32);
    // The standalone offers no registration (owner, 2026-10-19).
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::moved::image_bytes;
    use crate::strings::owner::strings;

    #[test]
    fn first_values_are_the_games() {
        let strings = strings();
        assert_eq!(
            image_bytes(0x4E075C, super::LOCALE_FILES.len()),
            super::LOCALE_FILES
        );
        assert_eq!(
            image_bytes(0x4E0784, super::KEYBOARD_FILES.len()),
            super::KEYBOARD_FILES
        );
        assert_eq!(
            image_bytes(0x4E0798, super::LOCALE_LEN),
            strings.language.locale.read()
        );
        assert_eq!(
            image_bytes(0x4F0CC8, 256),
            strings.language.plain_letters.read()
        );
    }
}
