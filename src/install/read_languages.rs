//! `install::read_languages` (0x413908): reads the languages file `lang.cfg`.
//!
//! In:  nothing
//! Out: every register preserved
//! A settings text with the keys `lang1` .. `lang9` (the table at 0x51C378: key, slot,
//! 0x7F), each value `name:directory`. Every key found has its value copied to its slot
//! of 0x80 bytes at 0x51BEEE (`sub_4139AB`, inlined). Then the slots that start with a
//! character above a space other than `:` and have a `:` are kept, moved together to the
//! front, the `:` made a NUL, and counted in `dword_51BEEA`; then `choose_language` for the
//! first of them. Without the file nothing changes, not even the count.
//! Callers: `start` and `sub_412DE0`.
//! Refuses a value of 0x80 bytes or more (the original writes on into the next slot).
//! Self-test: a redirected copy that reads a file of the test's.

use crate::bridge::Raw;
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, game_write, hook_asm, rva};
use crate::install::choose_language::{
    LANGUAGE_COUNT_RVA, LANGUAGE_SLOT, LANGUAGE_SLOTS, LANGUAGES_RVA, choose_language,
};
use crate::install::{LOCALE_RVA, PATH_ROOM, copy_reading};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, dispatch_state, restore};
use crate::settings::read_text::read_text;
use crate::strings::same_text::case_table;
use rc2k_core::install::config;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::read_languages", rva(0x413908), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const PATH_RVA: usize = rva(0x51C36E); // "lang.cfg"

// ---- Rust version ----

pub unsafe fn read_languages(path: *const u8) -> Result<(), BadInput> {
    unsafe {
        let Some((text, _)) = read_text(path)? else {
            return Ok(());
        };
        // Converted: `rc2k_core::install::config` (`find_languages`, its keys as constants;
        // `keep_languages`). Found first, stored after (`sub_4139AB`).
        let found = config::find_languages(&Raw::new(text), &case_table())?;
        let slots = std::slice::from_raw_parts_mut(
            game_addr(LANGUAGES_RVA) as *mut u8,
            LANGUAGE_SLOT * LANGUAGE_SLOTS,
        );
        for (slot, value) in found {
            let at = slot * LANGUAGE_SLOT;
            slots[at..at + value.len()].copy_from_slice(&value);
            slots[at + value.len()] = 0;
        }
        let kept = config::keep_languages(slots);
        game_write(LANGUAGE_COUNT_RVA, kept);
        choose_language(1)
    }
}

// ---- hook ----

fn regions() -> [Region; 3] {
    [
        at(LANGUAGE_COUNT_RVA),
        (
            game_addr(LANGUAGES_RVA) as u32,
            LANGUAGE_SLOT * LANGUAGE_SLOTS,
        ),
        (game_addr(LOCALE_RVA) as u32, PATH_ROOM),
    ]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { read_languages(game_addr(PATH_RVA) as *const u8) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

const LEN: usize = 0x72;
const CALLS: [usize; 3] = [0x07, 0x0E, 0x6B];
const TEST_FILE: &str = "var\\rc2k_hook_lang.cfg";
const TEST_PATH: &[u8] = b"var\\rc2k_hook_lang.cfg\0";
const FILES: [&[u8]; 7] = [
    b"lang1=English:english\r\nlang2=Deutsch:deutsch\r\nlang3=Portugues:portuges\r\n",
    b"lang3=Third:third\r\nlang1=nocolon\r\nlang2=:empty\r\nlang9=\"Ninth quoted:ninth dir\"\r\n",
    b"lang2=Second:second:more\r\nlang5 Fifth:\r\nlang7=\r\n",
    b"LANG1 = One:one\tignored\r\nlang4= :space\r\nlang6=Six:six\r\n",
    b"lang1=a:b\r\nlang2=c:d\r\nlang3=e:f\r\nlang4=g:h\r\nlang5=i:j\r\nlang6=k:l\r\nlang7=m:n\r\nlang8=o:p\r\nlang9=q:r\r\n",
    b"other=1\r\n",
    b"",
];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match unsafe { copy_reading(rva(0x413908), LEN, PATH_RVA, &CALLS, TEST_PATH) } {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let regions = regions();
    let saved = capture(&regions);
    let last = crate::files::stream::LastPath::save();
    let mut made = true;
    for file in FILES.iter().map(Some).chain([None]) {
        let _ = std::fs::remove_file(TEST_FILE);
        if let Some(file) = file
            && std::fs::write(TEST_FILE, file).is_err()
        {
            made = false;
            break;
        }
        // Slots that are empty, and slots that hold what an earlier reading left.
        for old in [false, true] {
            let before = Regs::SENTINEL;
            let run = |rust: bool| unsafe {
                restore(&regions, &vec![0u8; saved.len()]);
                if old {
                    for slot in [1usize, 4, 8] {
                        let text = format!("Old{slot}:old{slot}\0");
                        std::ptr::copy(
                            text.as_ptr(),
                            game_addr(LANGUAGES_RVA + slot * LANGUAGE_SLOT) as *mut u8,
                            text.len(),
                        );
                    }
                    game_write(LANGUAGE_COUNT_RVA, 5u32);
                }
                let done = if rust {
                    read_languages(TEST_PATH.as_ptr()).is_ok()
                } else {
                    let mut ro = before;
                    call_game(&mut ro, copy.addr());
                    ro.preserved_except(&before, 0)
                };
                (done, capture(&regions))
            };
            let (theirs, mine) = (run(false), run(true));
            t.check(theirs == mine && theirs.0, || {
                let first = theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b);
                format!(
                    "file={:?} old slots={old}: original done={} rust done={}; first different byte {first:?} (4 the count, 0x480 the slots, then the language)",
                    file.map(|file| String::from_utf8_lossy(file)),
                    theirs.0,
                    mine.0
                )
            });
        }
    }
    restore(&regions, &saved);
    last.restore();
    let _ = std::fs::remove_file(TEST_FILE);
    if !made {
        return t.skip("cannot create a test file in the game's var folder");
    }
    t.finish()
}
