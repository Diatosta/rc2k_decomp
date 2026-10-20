//! `settings::read_settings` (0x4114A6): reads `var\ral.cfg` into the game's settings.
//!
//! In:  nothing
//! Out: every register preserved
//! `read_text` on `var\ral.cfg`; if it reads, `sub_4114FF` (0x4114FF, one caller, here as
//! `apply_settings`): for each of the 77 entries of the table at 0x518C09 (a key, then the
//! address of its variable), if `find_key` finds the key, the variable becomes
//! `parse_number` of its value. A key that is not in the file leaves its variable alone;
//! a key without a number makes it 0. Two keys share a variable twice (`noaudio` and
//! `nosound`, `allsecretcars` and `secretcar`): the later one in the table wins.
//! Callers: `start` and `sub_411239`.
//! Self-test: `apply_settings` against the original's `sub_4114FF` on texts of its own,
//! and the whole function on the game's own file.

use crate::bridge::Raw;
use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, call_game, game_addr, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use crate::settings::read_text::read_text;
use crate::strings::same_text::case_table;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("settings::read_settings", rva(0x4114A6), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const PATH_RVA: usize = rva(0x5295CC); // "var\ral.cfg"
/// aNotaskswitch: the settings, each a key and the address of its dword; a 0 ends them.
const TABLE_RVA: usize = rva(0x518C09);
const TABLE_END_RVA: usize = rva(0x5190F1);
const APPLY_SETTINGS: usize = rva(0x4114FF); // sub_4114FF

// ---- Rust version ----

/// `sub_4114FF`: the settings a text names. Converted: `rc2k_core::settings::table` and
/// `apply`; here the variables (the table's dwords are their addresses) are stored.
unsafe fn settings() -> Result<Vec<(Vec<u8>, u32)>, BadInput> {
    let table = unsafe { Raw::new(game_addr(TABLE_RVA) as *const u8) };
    Ok(rc2k_core::settings::table(
        &table,
        TABLE_END_RVA - TABLE_RVA,
    )?)
}

pub unsafe fn apply_settings(text: *const u8) -> Result<(), BadInput> {
    if text.is_null() {
        return Err(BadInput("no settings text or no key"));
    }
    unsafe {
        let table = settings()?;
        let keys: Vec<&[u8]> = table.iter().map(|(key, _)| key.as_slice()).collect();
        // Found first, stored after: nothing is written if a key is refused.
        let found = rc2k_core::settings::apply(&Raw::new(text), &keys, &case_table())?;
        for (n, number) in found {
            (table[n].1 as *mut u32).write_unaligned(number);
        }
        Ok(())
    }
}

pub unsafe fn read_settings() -> Result<(), BadInput> {
    unsafe {
        match read_text(game_addr(PATH_RVA) as *const u8)? {
            Some((text, _)) => apply_settings(text),
            None => Ok(()),
        }
    }
}

// ---- hook ----

fn regions() -> Vec<Region> {
    let mut regions: Vec<Region> = unsafe { settings() }
        .unwrap_or_default()
        .into_iter()
        .map(|(_, variable)| (variable, 4))
        .collect();
    regions.sort();
    regions.dedup();
    regions
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { read_settings() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

/// Settings that nothing reads while the game stands still in a menu.
const TEST_TEXTS: [&[u8]; 6] = [
    b"\0",
    b"seasoncode=3\r\nsparksmode = 0x10\r\nreplaykeys\t-2\r\n\0",
    b"; comment\r\nSeasonCode=$ff\r\nsparksmodes=9\r\ninetport=+2300 \r\n\0",
    b"allsecretcars=1\r\nsecretcar=2\r\nlexcontrol\r\nframecounter=x\r\n\0",
    b"secretcar=2\r\nallsecretcars=1\r\nkey 99999999999\r\n\0",
    b"screensnapshot=1\nscreensnapshot=2\nwrreplayrate==5\n\0",
];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    if regions.is_empty() {
        return t.skip("the settings table is not what it was");
    }
    let saved = capture(&regions);
    let last = crate::files::stream::LastPath::save();
    for text in TEST_TEXTS {
        let before = Regs {
            esi: text.as_ptr() as u32,
            ..Regs::SENTINEL
        };
        let run = |rust: bool| unsafe {
            restore(&regions, &saved);
            let done = if rust {
                apply_settings(text.as_ptr()).is_ok()
            } else {
                let mut ro = before;
                call_game(&mut ro, game_addr(APPLY_SETTINGS));
                ro.preserved_except(&before, 0)
            };
            (done, capture(&regions))
        };
        let (theirs, mine) = (run(false), run(true));
        t.check(theirs == mine && theirs.0, || {
            let first = theirs.1.iter().zip(&mine.1).position(|(a, b)| a != b);
            format!(
                "text={:?}: original done={} rust done={}; first different setting {:x?}",
                String::from_utf8_lossy(text),
                theirs.0,
                mine.0,
                first.map(|byte| regions[byte / 4].0)
            )
        });
    }
    // The game's own file, as it is.
    check_state(
        &mut t,
        "var\\ral.cfg",
        call_original,
        Regs::SENTINEL,
        &|| restore(&regions, &saved),
        &regions,
        &rust,
    );
    restore(&regions, &saved);
    last.restore();
    t.finish()
}
