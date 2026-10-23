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
use crate::framework::{game_addr, game_read, game_write, rva};
use crate::input::key_to_char::{LAYOUT_A_RVA, LAYOUT_RVA};
use crate::settings::find_key::LOWER_RVA;
use crate::strings::append_string_by_id::push_string;
use crate::strings::text_from_ids::text;
use rc2k_core::strings::case::CaseTable;
use rc2k_core::strings::language::Locale;
use windows_sys::Win32::Globalization::GetSystemDefaultLCID;

// ---- game data ----

const LEXICON_RVA: usize = rva(0x4E07F8); // g_lexicon
const LEXICON_ROOM: usize = 0x4F07C8 - 0x4E07F8;
const STRING_TABLE_RVA: usize = rva(0x4E07D4); // g_stringTable
const BUILT_IN_LAYOUT_RVA: usize = rva(0x4F7FDC); // unk_4F7FDC
const LOCALE_FILES_RVA: usize = rva(0x4E075C); // "files\%locale\*.loc"
const KEYBOARD_FILES_RVA: usize = rva(0x4E0784); // "files\%locale\*.kim"
/// dword_4E0798: the `.loc` file's 0x30 bytes. +0 "LOCL", +8 the notice kind, +0xF units.
const LOCALE_RVA: usize = rva(0x4E0798);
const LOCALE_LEN: usize = 0x30;
/// dword_4F07C8: the `.kim` file, 0x400 bytes. +0 "KEYB", +8 a kind, +9 and +0xA two modes,
/// +0xB the layout's number, +0xC the layout chosen (written here), +0x30 the layouts.
const KEYBOARD_RVA: usize = rva(0x4F07C8);
const KEYBOARD_LEN: usize = 0x400;
const UNITS_RVA: usize = rva(0x51DE2C); // g_isImperialUnits
/// Each car's copy of the units, at its +0x178.
const CAR_UNITS: [usize; 4] = [rva(0x755BC4), rva(0x755E50), rva(0x7560DC), rva(0x756368)];
const PLAIN_LETTERS_RVA: usize = rva(0x4F0CC8); // byte_4F0CC8
const KEYBOARD_TYPE_RVA: usize = rva(0x518404); // dword_518404
const NEC_KEYBOARD_RVA: usize = rva(0x4F0DE0); // aKeyb: a built-in file head, and
const NEC_LAYOUT_RVA: usize = rva(0x4F0E10); // unk_4F0E10: its layout
const NEC_MARK_RVA: usize = rva(0x51BAE4); // dword_51BAE4
const TABLES_UNITS_RVA: usize = rva(0x51B980); // dword_51B980 (`bundleheapsize`)
const REGISTRATION_RVA: usize = rva(0x51B9E4); // dword_51B9E4

// ---- Rust version ----

fn fatal(code: u32, message: &'static str) -> Refused {
    FatalError { code, message }.into()
}

/// String `id` as the original builds it: in the text buffer. Returns its bytes.
unsafe fn built(id: u32, with_nul: bool) -> Result<Vec<u8>, BadInput> {
    unsafe {
        let mut bytes = Vec::new();
        push_string(&mut bytes, id);
        if bytes.len() >= 0x100 {
            return Err(BadInput(
                "a string of the language's set-up longer than expected",
            ));
        }
        std::ptr::copy(bytes.as_ptr(), text(), bytes.len());
        if with_nul {
            text().add(bytes.len()).write(0);
        }
        Ok(bytes)
    }
}

/// `sub_4034C2` (converted: `rc2k_core::strings::case::CaseTable::{for_language, plain_letters}`).
unsafe fn make_case_tables() -> Result<(), BadInput> {
    unsafe {
        let lower = CaseTable::for_language(&built(2, true)?)?;
        std::ptr::copy(lower.lower.as_ptr(), game_addr(LOWER_RVA) as *mut u8, 256);
        let plain = lower.plain_letters(&built(3, true)?);
        std::ptr::copy(
            plain.lower.as_ptr(),
            game_addr(PLAIN_LETTERS_RVA) as *mut u8,
            256,
        );
        Ok(())
    }
}

/// `sub_416389` (converted: `rc2k_core::strings::lexicon::correct`), on the lexicon's room.
unsafe fn correct_lexicon() -> Result<(), BadInput> {
    let table =
        unsafe { std::slice::from_raw_parts_mut(game_addr(LEXICON_RVA) as *mut u8, LEXICON_ROOM) };
    Ok(rc2k_core::strings::lexicon::correct(table)?)
}

/// `compile` stands for `strings::compile_lexicon`.
pub unsafe fn load_language(
    compile: &mut dyn FnMut() -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        compile()?;
        game_write(STRING_TABLE_RVA, game_addr(LEXICON_RVA) as u32);
        game_write(LAYOUT_RVA, game_addr(BUILT_IN_LAYOUT_RVA) as u32);
        game_write(KEYBOARD_RVA + 9, 1u8);
        game_write(KEYBOARD_RVA + 0xA, 1u8);

        let locale = game_addr(LOCALE_RVA) as *mut u8;
        let pattern = game_addr(LOCALE_FILES_RVA) as *const u8;
        if read_first_match(pattern, locale, LOCALE_LEN as u32, 0)?.is_none() {
            return Err(fatal(0x80E3_8381, ""));
        }
        // Converted: `rc2k_core::strings::language::Locale`.
        let Ok(file) = Locale::parse(&game_read::<[u8; LOCALE_LEN]>(LOCALE_RVA)) else {
            return Err(fatal(0x80E3_8382, ""));
        };
        let units = file.units as u32;
        game_write(UNITS_RVA, units);
        for car in CAR_UNITS {
            game_write(car, units);
        }

        let keyboard = game_addr(KEYBOARD_RVA) as *mut u8;
        let pattern = game_addr(KEYBOARD_FILES_RVA) as *const u8;
        if read_first_match(pattern, keyboard, KEYBOARD_LEN as u32, 0)?.is_some() {
            if &game_read::<[u8; 4]>(KEYBOARD_RVA) != b"KEYB" {
                return Err(fatal(0x80E3_83A2, ""));
            }
            let layouts = game_addr(LAYOUT_A_RVA) as u32;
            for n in 0..10 {
                let offset: u32 = game_read(LAYOUT_A_RVA + n * 4);
                if offset != 0 {
                    game_write(LAYOUT_A_RVA + n * 4, offset.wrapping_add(layouts));
                }
            }
            let chosen = game_read::<u8>(KEYBOARD_RVA + 0xB) as u32 * 0x14;
            let layout = layouts.wrapping_add(chosen);
            game_write(LAYOUT_RVA, layout);
            if game_read::<u8>(KEYBOARD_RVA + 8) == 0x0C {
                let fourth = ((layout + 0x10) as *const u32).read_unaligned() as *mut u8;
                if fourth.is_null() {
                    return Err(BadInput("a keyboard of kind 0x0C without its fourth table").into());
                }
                for (at, value) in [(0x34, 0x1034u16), (0x36, 0x1035), (0x52, 0x1036)] {
                    (fourth.add(at) as *mut u16).write_unaligned(value);
                }
            }
        }

        make_case_tables()?;

        // `sub_403462`
        if game_read::<u32>(KEYBOARD_TYPE_RVA) == 2 {
            std::ptr::copy(game_addr(NEC_KEYBOARD_RVA) as *const u8, keyboard, 0x30);
            game_write(LAYOUT_RVA, game_addr(NEC_LAYOUT_RVA) as u32);
            game_write(NEC_MARK_RVA, 0x61u32);
        }

        // `sub_403495`
        let kind = file.notice_kind;
        if GetSystemDefaultLCID() & 0x3FF == 0x11 && kind != 7 {
            return Err(fatal(
                0x9000_6044,
                "You cannot run this version of RC in Japan\t",
            ));
        }
        if kind == 7 {
            game_write(TABLES_UNITS_RVA, 0x2EEu32);
        }

        correct_lexicon()?;

        game_write(REGISTRATION_RVA, 0u32);
        // The standalone offers no registration (owner, 2026-10-19).
        Ok(())
    }
}
