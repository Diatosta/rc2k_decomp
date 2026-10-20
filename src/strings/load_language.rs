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
use crate::files::file_exists::file_exists;
use crate::files::read_first_match::read_first_match;
use crate::framework::{
    RawHook, Regs, SelfTest, call_game, game_addr, game_read, game_write, hook_asm, rva, shadow,
};
use crate::input::key_to_char::{LAYOUT_A_RVA, LAYOUT_RVA};
use crate::menu::frame_test;
use crate::redirect::RedirectedCopy;
use crate::service::harness::{Region, at, capture, restore};
use crate::settings::find_key::LOWER_RVA;
use crate::strings::append_string_by_id::push_string;
use crate::strings::compile_lexicon::{compile_lexicon, compiled};
use crate::strings::text_from_ids::text;
use rc2k_core::case::CaseTable;
use rc2k_core::language::Locale;
use std::sync::atomic::AtomicUsize;
use windows_sys::Win32::Globalization::GetSystemDefaultLCID;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("strings::load_language", rva(0x40331F), thunk, &ORIGINAL)
    .private_copy(0x143)
    .relocate_calls(&[
        0x01, 0x3B, 0x40, 0x50, 0x89, 0x9A, 0xED, 0xF2, 0xF7, 0xFC, 0x121, 0x130,
    ])
    .manual_patch()
    .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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
const REGISTRATION_PROGRAM_RVA: usize = rva(0x51B9EC); // ";4ereg\eregip.exe"
const COMPILE_LEXICON: usize = rva(0x40356E);

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

/// `sub_4034C2` (converted: `rc2k_core::case::CaseTable::{for_language, plain_letters}`).
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

/// `sub_416389` (converted: `rc2k_core::lexicon::correct`), on the lexicon's room.
unsafe fn correct_lexicon() -> Result<(), BadInput> {
    let table =
        unsafe { std::slice::from_raw_parts_mut(game_addr(LEXICON_RVA) as *mut u8, LEXICON_ROOM) };
    Ok(rc2k_core::lexicon::correct(table)?)
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
        // Converted: `rc2k_core::language::Locale`.
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
        if kind == 2
            && !crate::framework::standalone()
            && !built(0x57C, false)?.is_empty()
            && file_exists(game_addr(REGISTRATION_PROGRAM_RVA) as *const u8)?
        {
            game_write(REGISTRATION_RVA, 1u32);
        }
        Ok(())
    }
}

// ---- hook ----

/// What the function writes, but for the lexicon; `text` is the text buffer's start.
fn small_regions() -> Vec<Region> {
    let mut regions = vec![
        at(STRING_TABLE_RVA),
        (game_addr(LOCALE_RVA) as u32, LOCALE_LEN),
        (game_addr(KEYBOARD_RVA) as u32, KEYBOARD_LEN),
        (game_addr(LOWER_RVA) as u32, 0x100),
        (game_addr(PLAIN_LETTERS_RVA) as u32, 0x100),
        at(UNITS_RVA),
        at(NEC_MARK_RVA),
        at(TABLES_UNITS_RVA),
        at(REGISTRATION_RVA),
        (text() as u32, 0x100),
    ];
    regions.extend(CAR_UNITS.map(at));
    regions
}

/// With the lexicon's whole place, and the allocator's three figures.
fn all_regions() -> Vec<Region> {
    let mut regions = small_regions();
    regions.push((game_addr(LEXICON_RVA) as u32, LEXICON_ROOM));
    regions.push((game_addr(crate::memory::MOST_HELD_RVA) as u32, 12));
    regions
}

/// Where two captures of `regions` first differ: (the region's address, the byte in it).
fn first_difference(regions: &[Region], a: &[u8], b: &[u8]) -> Option<(u32, usize)> {
    let mut byte = a.iter().zip(b).position(|(a, b)| a != b)?;
    for &(address, len) in regions {
        if byte < len {
            return Some((address, byte));
        }
        byte -= len;
    }
    None
}

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    let mut compile = || unsafe { compile_lexicon() };
    if !shadow() {
        if let Err(e) = unsafe { load_language(&mut compile) } {
            unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) };
        }
        return;
    }
    let regions = all_regions();
    let before = capture(&regions);
    let mine = unsafe { load_language(&mut compile) };
    let left = capture(&regions);
    restore(&regions, &before);
    match mine {
        Ok(()) => {
            let mut o = *r;
            unsafe { call_original(&mut o) };
            let theirs = capture(&regions);
            if theirs != left || !o.preserved_except(r, 0) {
                HOOK.mismatch(|| {
                    format!(
                        "first difference (region, byte): {:x?}",
                        first_difference(&regions, &theirs, &left)
                    )
                });
            }
        }
        Err(e) => unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) },
    }
}

// ---- self-test ----

const LEN: usize = 0x143;
const ENTRY_BYTES: [u8; 8] = [0x60, 0xE8, 0x49, 0x02, 0x00, 0x00, 0xC7, 0x05];
const CALLS: [usize; 9] = [0x01, 0x3B, 0x89, 0xED, 0xF2, 0xF7, 0xFC, 0x121, 0x130];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // A lexicon as the compiler leaves it, before the corrections.
    let fresh = match unsafe { compiled() } {
        Ok(compiled) => compiled.table,
        Err(_) => return t.skip("the language file does not compile"),
    };
    let copy = unsafe {
        RedirectedCopy::new(
            game_addr(rva(0x40331F)),
            LEN,
            &ENTRY_BYTES,
            &CALLS,
            &|callee| (callee == COMPILE_LEXICON).then(|| frame_test::stand_in(0)),
        )
    };
    let copy = match copy {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let lexicon: Region = (game_addr(LEXICON_RVA) as u32, fresh.len());
    let small = small_regions();
    let mut regions = small.clone();
    regions.push(lexicon);
    let saved = capture(&regions);
    let last = crate::files::stream::LastPath::save();
    let search = capture(&[(game_addr(rva(0x4E05A4)) as u32, 0x144)]);

    let before = Regs::SENTINEL;
    let run = |rust: bool| unsafe {
        restore(
            &small,
            &vec![0xAA; small.iter().map(|region| region.1).sum()],
        );
        // The text buffer is never left without an end.
        text().add(0xFF).write(0);
        std::ptr::copy(
            fresh.as_ptr(),
            game_addr(LEXICON_RVA) as *mut u8,
            fresh.len(),
        );
        game_write(STRING_TABLE_RVA, game_addr(LEXICON_RVA) as u32);
        frame_test::begin(u32::MAX, 0);
        let done = if rust {
            use crate::menu::screen_loop::Env;
            let mut entry = before;
            load_language(&mut || {
                frame_test::Noting {
                    places: &[(COMPILE_LEXICON, 0)],
                }
                .call(COMPILE_LEXICON, &mut entry);
                Ok(())
            })
            .is_ok()
        } else {
            let mut ro = before;
            call_game(&mut ro, copy.addr());
            ro.preserved_except(&before, 0)
        };
        (done, frame_test::notes(), capture(&regions))
    };
    let (theirs, mine) = (run(false), run(true));
    t.check(theirs == mine && theirs.0 && theirs.1.len() == 1, || {
        format!(
            "a fresh lexicon: original done={} compiles={}; rust done={} compiles={}; first difference (region, byte) {:x?}",
            theirs.0,
            theirs.1.len(),
            mine.0,
            mine.1.len(),
            first_difference(&regions, &theirs.2, &mine.2)
        )
    });
    // And what the game has had since its start is that too: the lexicon with its
    // corrections, the locale, the two case tables.
    let stable = [
        lexicon,
        (game_addr(LOCALE_RVA) as u32, LOCALE_LEN),
        (game_addr(LOWER_RVA) as u32, 0x100),
        (game_addr(PLAIN_LETTERS_RVA) as u32, 0x100),
    ];
    let now = capture(&stable);
    restore(&regions, &saved);
    restore(&[(game_addr(rva(0x4E05A4)) as u32, 0x144)], &search);
    last.restore();
    let had = capture(&stable);
    t.check(now == had, || {
        format!(
            "the game's own tables against a fresh load: first difference (region, byte) {:x?}",
            first_difference(&stable, &had, &now)
        )
    });
    t.finish()
}
