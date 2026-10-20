//! `settings::read_driver_names` (0x4162DB): other names for the game's drivers, from
//! `var\name.cfg`.
//!
//! In:  nothing
//! Out: every register preserved
//! A settings text with the keys `name000` .. `name179` (the key is made in place: the
//! number goes over the digits of `aName000`), each followed by up to three words as
//! `system::next_argument` cuts them: a name, of which at most 30 characters are kept in
//! the pool at `unk_95EFC8` (one after another, each with a NUL) and pointed at by
//! `dword_95ECF8[n]`; a number under 0x28 for the driver's record at `dword_73B688` and a
//! number under 0x22 for `dword_73B68C` (records of 0x94 bytes). A number that is too
//! large is left out; a missing third word ends the entry. Without the file nothing
//! changes. One caller, `sub_411239`. The owner's install has no such file.
//! Refuses names that together pass the pool's end at `unk_960000` (the original writes
//! on; 180 names of 30 characters would).
//! Self-test: a redirected copy that reads a file of the test's.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, game_write, hook_asm, rva};
use crate::install::copy_reading;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, dispatch_state, restore};
use crate::settings::read_text::read_text;
use crate::strings::format_u32::format_u32;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "settings::read_driver_names",
    rva(0x4162DB),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const PATH_RVA: usize = rva(0x529D98); // "var\name.cfg"
const KEY_RVA: usize = rva(0x529DA5); // aName000
const DRIVERS: usize = 180;
const NAMES_RVA: usize = rva(0x95ECF8); // dword_95ECF8: a pointer a driver
const POOL_RVA: usize = rva(0x95EFC8); // unk_95EFC8
/// Up to `unk_960000`: room for about 130 names of the full length, not for 180.
const POOL_ROOM: usize = 0x960000 - 0x95EFC8;
const RECORD_LEN: usize = 0x94;
const FIRST_NUMBER_RVA: usize = rva(0x73B688); // dword_73B688
const SECOND_NUMBER_RVA: usize = rva(0x73B68C); // dword_73B68C

// ---- Rust version ----

pub unsafe fn read_driver_names(path: *const u8) -> Result<(), BadInput> {
    // Converted: `rc2k_core::files::settings::driver_names`; the pool's room checked before
    // anything is written. The key the original makes in place (`aName000`) is left as
    // its last one, "name179".
    unsafe {
        let Some((text, _)) = read_text(path)? else {
            return Ok(());
        };
        let entries = rc2k_core::files::settings::driver_names(
            &crate::bridge::Raw::new(text),
            &crate::strings::same_text::case_table(),
        )
        .map_err(|e| BadInput(e.0))?;
        let room: usize = entries
            .iter()
            .flatten()
            .filter_map(|e| e.name.as_ref())
            .map(|n| n.len() + 1)
            .sum();
        if room > POOL_ROOM {
            return Err(BadInput("more driver names than their pool holds"));
        }
        let last = format_u32(DRIVERS as u32 - 1, 0x13);
        std::ptr::copy(
            last.as_ptr(),
            (game_addr(KEY_RVA) as *mut u8).add(4),
            last.len(),
        );
        let mut pool = game_addr(POOL_RVA) as *mut u8;
        for (driver, entry) in entries.iter().enumerate() {
            let Some(entry) = entry else {
                continue;
            };
            if let Some(name) = &entry.name {
                game_write(NAMES_RVA + driver * 4, pool as u32);
                std::ptr::copy(name.as_ptr(), pool, name.len());
                pool.add(name.len()).write(0);
                pool = pool.add(name.len() + 1);
            }
            if let Some(n) = entry.first {
                game_write(FIRST_NUMBER_RVA + driver * RECORD_LEN, n);
            }
            if let Some(n) = entry.second {
                game_write(SECOND_NUMBER_RVA + driver * RECORD_LEN, n);
            }
        }
        Ok(())
    }
}

// ---- hook ----

fn regions() -> Vec<Region> {
    vec![
        (game_addr(KEY_RVA) as u32, 8),
        (game_addr(NAMES_RVA) as u32, DRIVERS * 4),
        (game_addr(POOL_RVA) as u32, POOL_ROOM),
        // The two numbers of every record, with what lies between them.
        (
            game_addr(FIRST_NUMBER_RVA) as u32,
            (DRIVERS - 1) * RECORD_LEN + 8,
        ),
    ]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { read_driver_names(game_addr(PATH_RVA) as *const u8) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

const LEN: usize = 0xAE;
const CALLS: [usize; 8] = [0x07, 0x27, 0x32, 0x39, 0x5F, 0x68, 0x80, 0x89];
const TEST_FILE: &str = "var\\rc2k_hook_name.cfg";
const TEST_PATH: &[u8] = b"var\\rc2k_hook_name.cfg\0";
const FILES: [&[u8]; 6] = [
    b"name000=Colin 5 3\r\nname001 = \"Two Words\" 39 33\r\nname179=Last 0 0\r\n",
    b"name002=OnlyAName\r\nname003=Name 7\r\nname004=Big 40 34\r\nname005=\"\" 2 1\r\n",
    b"name010=AVeryLongNameThatGoesOnWellPastThirtyCharacters 1 2\r\nname011=x\t3\t4\r\n",
    b"NAME020=Upper 0x10 $11\r\nname021=neg -1 -2\r\nname180=Beyond 1 1\r\nname22=short 1 1\r\n",
    b"; nothing here\r\nother=1\r\n",
    b"",
];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let copy = match unsafe { copy_reading(rva(0x4162DB), LEN, PATH_RVA, &CALLS, TEST_PATH) } {
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
        let before = Regs::SENTINEL;
        let run = |rust: bool| unsafe {
            restore(&regions, &vec![0xAA; saved.len()]);
            std::ptr::copy(
                c"name000".as_ptr().cast::<u8>(),
                game_addr(KEY_RVA) as *mut u8,
                8,
            );
            let done = if rust {
                read_driver_names(TEST_PATH.as_ptr()).is_ok()
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
                "file={:?}: original done={} rust done={}; first different byte {first:?} (8 the key, 0x2D0 the pointers, 0x1038 the pool, then the records)",
                file.map(|file| String::from_utf8_lossy(&file[..file.len().min(50)])),
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
