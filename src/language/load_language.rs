//! Makes the language ready: its strings, its units, its keyboard and its letters' cases.
//!
//! In this order:
//! - `language::compile_lexicon`; the keyboard layout (`input::layouts`) is the built-in one, with
//!   the layout flags both 1;
//! - the first `files\%locale\*.loc`, 0x30 bytes: none is fatal error 0x80E38381, one that does
//!   not start "LOCL" 0x80E38382. Its byte +0xF is the units (the units setting and each car's
//!   +0x178); its byte +8 is read further down;
//! - the language's built-in own texts (`language::own_texts`);
//! - the first `files\%locale\*.kim`, up to 0x400 bytes, if there is one: not starting "KEYB" is
//!   fatal error 0x80E383A2. The ten offsets at its +0x30 become addresses, the layout is the
//!   block of 0x14 bytes its byte +0xB chooses, and if its byte +8 is 0x0C three entries of that
//!   layout's fourth table are set (0x1034, 0x1035, 0x1036 at +0x34, +0x36, +0x52);
//! - `strings::make_case_tables`: the case tables. The lower-case table: every character
//!   itself, A .. Z in lower case, then string 2 as pairs "lower, upper". The plain-letters
//!   table: a copy of that in which string 3's accented letters become the plain letter before
//!   them ("aàáâ..");
//! - with keyboard type 2 (`keyboard_type`) a built-in layout takes the file's place;
//! - on a Japanese system (language 0x11) a notice kind other than 7 is fatal error 0x90006044;
//!   with kind 7 the glyph tables' units (`glyph_tables.units`) are 0x2EE;
//! - `strings::correct_lexicon`: four corrections in the lexicon itself: in string 0x6CA the word
//!   "Bowland" loses its `w`, in 0x6AB "Laukennen" becomes "Laukkanen", and in 0x23B the word
//!   "A7" or "(A7)" is replaced by string 0x23A's "A8" or "(A8)";
//!
//! Refuses a string 2 of odd length and one of the four corrections meeting its word where the
//! string is that word alone.

use crate::errors::{BadInput, FatalError, Refused};
use crate::files::read_first_match::read_first_match;

use crate::strings::append_string_by_id::push_string;
use crate::strings::owner::Text;
use crate::strings::same_text::set_case_table;
use rc2k_core::strings::case::CaseTable;
use rc2k_core::strings::language::Locale;

const LOCALE_FILES: &[u8] = b"files\\%locale\\*.loc";
const KEYBOARD_FILES: &[u8] = b"files\\%locale\\*.kim";
fn fatal(code: u32, message: &'static str) -> Refused {
    FatalError { code, message }.into()
}

/// String `id` built in the text buffer. Returns its bytes.
fn built(
    o: crate::owners::Owners<'static>,
    text: &Text,
    id: u32,
    with_nul: bool,
) -> Result<Vec<u8>, BadInput> {
    let mut bytes = Vec::new();
    push_string(
        crate::strings::append_string_by_id::StringOwners::of(o),
        &mut bytes,
        id,
    )?;
    if bytes.len() >= 0x100 {
        return Err(BadInput(
            "a string of the language's set-up longer than expected",
        ));
    }
    let buffer = &text.buffer.text;
    buffer.write_at(0, &bytes, "the text buffer")?;
    if with_nul {
        buffer.write_at(bytes.len(), &[0], "the text buffer")?;
    }
    Ok(bytes)
}

/// `strings::make_case_tables` (`rc2k_core::strings::case::CaseTable::{for_language,
/// plain_letters}`).
fn make_case_tables(o: crate::owners::Owners<'static>) -> Result<(), BadInput> {
    let strings = o.strings;
    let lower = CaseTable::for_language(&built(o, &strings.text, 2, true)?)?;
    set_case_table(&lower.lower);
    let plain = lower.plain_letters(&built(o, &strings.text, 3, true)?);
    strings.language.plain_letters.write(&plain.lower);
    Ok(())
}

/// `strings::correct_lexicon` (`rc2k_core::strings::lexicon::correct`), on the lexicon's room.
fn correct_lexicon(strings: &crate::strings::owner::Strings) -> Result<(), BadInput> {
    let mut table = crate::strings::string_table::lexicon_bytes(strings)?;
    rc2k_core::strings::lexicon::correct(&mut table)?;
    crate::strings::string_table::write_lexicon(strings, &table)?;
    crate::strings::string_table::publish(strings)
}

/// `compile` stands for `language::compile_lexicon`.
pub fn load_language(
    o: crate::owners::Owners<'static>,
    compile: &mut dyn FnMut() -> Result<(), Refused>,
) -> Result<(), Refused> {
    let strings = o.strings;
    compile()?;
    o.input.layouts().use_built_in(o.input);

    let mut locale = strings.language.locale.read();
    let found = read_first_match(&o.system.work().files, LOCALE_FILES, &mut locale, 0)?;
    strings.language.locale.write(&locale);
    if found.is_none() {
        return Err(fatal(0x80E3_8381, ""));
    }
    let Ok(file) = Locale::parse(&strings.language.locale.read()) else {
        return Err(fatal(0x80E3_8382, ""));
    };
    let units = file.units as u32;
    crate::strings::units::set_units(strings, units);
    // Each car's copy of the units, at its +0x178.
    for player in &o.menus.play.slots.records {
        player.update(|p| p.metric_units = units);
    }

    crate::language::own_texts::load(strings)?;

    // Read over the block the last load left (a shorter file leaves its tail).
    let mut keyboard = o.input.layouts().file_bytes();
    let found = read_first_match(&o.system.work().files, KEYBOARD_FILES, &mut keyboard, 0)?;
    if found.is_some() {
        if &keyboard[..4] != b"KEYB" {
            return Err(fatal(0x80E3_83A2, ""));
        }
        o.input
            .layouts()
            .use_file(rc2k_core::input::layouts::parse(&keyboard)?);
    }

    make_case_tables(o)?;

    let kind = file.notice_kind;
    if crate::platform::os::language_id() & 0x3FF == 0x11 && kind != 7 {
        return Err(fatal(
            0x9000_6044,
            "You cannot run this version of RC in Japan\t",
        ));
    }
    if kind == 7 {
        o.gfx.fonts.glyph_tables.units.set(0x2EEu32);
    }

    correct_lexicon(strings)?;

    Ok(())
}
