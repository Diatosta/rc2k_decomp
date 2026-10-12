//! `install::read_record` (0x4110C9): reads the installation's record `var\ral.zog`.
//!
//! In:  nothing
//! Out: every register preserved
//! A settings text with three keys, each taken if it is there: `cdrom` becomes base
//! directory A (`byte_5189A8`), `installation` becomes `g_mediaMode` (0x5189A4), held to
//! 0 .. 3, and `language` becomes what `%` stands for in a path (`byte_518AA8`). The two
//! texts get a `\` after them unless they are empty or end in `\` or `/`. Without the
//! file nothing changes.
//! Callers: `start`, `sub_412DE0`, `sub_416653`.
//! Refuses a text of 0xFE bytes or more (the original writes past the 0x100 bytes).
//! Self-test: a redirected copy that reads a file of the test's.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, game_write, hook_asm, rva};
use crate::install::{BASE_DIRECTORY_RVA, LOCALE_RVA, PATH_ROOM, copy_reading};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, dispatch_state, restore};
use crate::settings::copy_value::value_at;
use crate::settings::find_key::find_key;
use crate::settings::parse_number::parse_number;
use crate::settings::read_text::read_text;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::read_record", rva(0x4110C9), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const PATH_RVA: usize = rva(0x5295C0); // "var\ral.zog"
const CDROM_KEY_RVA: usize = rva(0x518BED); // "cdrom"
const LANGUAGE_KEY_RVA: usize = rva(0x518BF3); // "language"
const INSTALLATION_KEY_RVA: usize = rva(0x518BFC); // "installation"
const MEDIA_MODE_RVA: usize = rva(0x5189A4); // g_mediaMode

// ---- Rust version ----

/// A directory as the record names it: the value, a NUL, and a `\` if it needs one.
unsafe fn directory(value: *const u8) -> Result<Vec<u8>, BadInput> {
    let mut text = unsafe { value_at(value) };
    if text.len() + 2 > PATH_ROOM {
        return Err(BadInput("a directory in the record longer than its place"));
    }
    if !matches!(text.last(), None | Some(b'/' | b'\\')) {
        text.push(b'\\');
    }
    text.push(0);
    Ok(text)
}

pub unsafe fn read_record(path: *const u8) -> Result<(), BadInput> {
    unsafe {
        let Some((text, _)) = read_text(path)? else {
            return Ok(());
        };
        let key = |key: usize| find_key(text, game_addr(key) as *const u8);
        let base = key(CDROM_KEY_RVA)?.map(|v| directory(v)).transpose()?;
        let mode = key(INSTALLATION_KEY_RVA)?.map(|v| (parse_number(v) as i32).clamp(0, 3));
        let locale = key(LANGUAGE_KEY_RVA)?.map(|v| directory(v)).transpose()?;
        for (place, text) in [(BASE_DIRECTORY_RVA, base), (LOCALE_RVA, locale)] {
            if let Some(text) = text {
                std::ptr::copy(text.as_ptr(), game_addr(place) as *mut u8, text.len());
            }
        }
        if let Some(mode) = mode {
            game_write(MEDIA_MODE_RVA, mode);
        }
        Ok(())
    }
}

// ---- hook ----

fn regions() -> [Region; 3] {
    [
        (game_addr(BASE_DIRECTORY_RVA) as u32, PATH_ROOM),
        (game_addr(LOCALE_RVA) as u32, PATH_ROOM),
        at(MEDIA_MODE_RVA),
    ]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { read_record(game_addr(PATH_RVA) as *const u8) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

const LEN: usize = 0xBD;
const CALLS: [usize; 9] = [0x07, 0x18, 0x25, 0x2A, 0x57, 0x5E, 0x82, 0x8F, 0x94];
const TEST_FILE: &str = "var\\rc2k_hook_record.zog";
const TEST_PATH: &[u8] = b"var\\rc2k_hook_record.zog\0";
const RECORDS: [&[u8]; 9] = [
    b"; Rally Championship\r\n; INSTALLATION CONFIGURATION\r\n\r\ninstallation=0\r\n",
    b"cdrom=D:\\\r\nlanguage=english\r\ninstallation=2\r\n",
    b"cdrom=\"E:\\rally disc\"\r\ninstallation=7\r\nlanguage=\"files\\loc/\"\r\n",
    b"installation=-4\r\ncdrom=\r\nlanguage\r\n",
    b"CDROM d:\\games\tignored\r\nInstallation $2\r\n",
    b"language=portuges\\\r\n",
    b"cdrom=x\r\n",
    b"nothing=here\r\n",
    b"",
];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match unsafe { copy_reading(rva(0x4110C9), LEN, PATH_RVA, &CALLS, TEST_PATH) } {
        Ok(copy) => copy,
        Err(why) => return t.skip(why),
    };
    let regions = regions();
    let saved = capture(&regions);
    let last = crate::files::stream::LastPath::save();
    let mut made = true;
    // The last case is the file not being there.
    for record in RECORDS.iter().map(Some).chain([None]) {
        let _ = std::fs::remove_file(TEST_FILE);
        if let Some(record) = record
            && std::fs::write(TEST_FILE, record).is_err()
        {
            made = false;
            break;
        }
        let before = Regs::SENTINEL;
        let run = |rust: bool| unsafe {
            restore(&regions, &vec![0xAA; saved.len()]);
            game_write(MEDIA_MODE_RVA, 1u32);
            let done = if rust {
                read_record(TEST_PATH.as_ptr()).is_ok()
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
                "record={:?}: original done={} rust done={}; first different byte {first:?} (0x100 the base directory, 0x100 the language, then the mode)",
                record.map(|record| String::from_utf8_lossy(record)),
                theirs.0,
                mine.0
            )
        });
    }
    restore(&regions, &saved);
    last.restore();
    let _ = std::fs::remove_file(TEST_FILE);
    if !made {
        return t.skip("cannot create a test file in the game's var folder");
    }
    t.finish()
}
