//! `install::write_registry` (0x413DC5): the installation written to the registry, under
//! `HKEY_LOCAL_MACHINE\SOFTWARE\Magnetic Fields\RC99`: the value `status` (one byte,
//! `g_installStatus`, type binary) and, if `g_installSource` is not empty, `source`
//! (`dword_51BC90` bytes, type string). The key opened (`RegCreateKeyA`) and closed
//! (`RegCloseKey`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//! A failed create, or a failed set of either value, is fatal error 0x80108420, 0x80108421
//! or 0x80108425. Three callers (`install::install_files`, `uninstall`, and `sub_412DE0`).
//! The registry calls through `api` (`installer::game_api` in the game: `windows-sys`
//! `Win32_System_Registry`); the standalone makes them too (section 15). Shadow mode runs
//! the original alone (a second write is harmless but needless). Self-test: a copy whose
//! three calls go to stand-ins (one made to fail at a time), against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{
    Api, REG_CLOSE_KEY, REG_CREATE_KEY, REG_KEY, REG_SET_VALUE, SOURCE, SOURCE_SIZE, STATUS, a,
    both, fatal, g, gb, quiet, rd,
};
use crate::race::viewer::{dispatch_parts, regs_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::write_registry", rva(0x413DC5), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const KEY_NAME: usize = 0x51BB60; // "SOFTWARE\Magnetic Fields\RC99"
const STATUS_NAME: usize = 0x51BB7E; // "status"
const SOURCE_NAME: usize = 0x51BB85; // "source"
const HKLM: u32 = 0x8000_0002;
const REG_SZ: u32 = 1;
const REG_BINARY: u32 = 3;

pub fn write_registry(entry: &Regs, api: Api) -> Result<Regs, Refused> {
    let key_at = a(REG_KEY);
    if api(REG_CREATE_KEY, &[HKLM, a(KEY_NAME), key_at]) != 0 {
        return Err(fatal(0x8010_8420));
    }
    let key = rd(key_at);
    if api(
        REG_SET_VALUE,
        &[key, a(STATUS_NAME), 0, REG_BINARY, a(STATUS), 1],
    ) != 0
    {
        return Err(fatal(0x8010_8421));
    }
    if gb(SOURCE) != 0
        && api(
            REG_SET_VALUE,
            &[key, a(SOURCE_NAME), 0, REG_SZ, a(SOURCE), g(SOURCE_SIZE)],
        ) != 0
    {
        return Err(fatal(0x8010_8425));
    }
    api(REG_CLOSE_KEY, &[key]);
    Ok(*entry)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        write_registry(r, &mut crate::install::installer::game_api)
    });
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| {
        write_registry(r, &mut crate::install::installer::game_api)
    })
}

/// The three imports, as stand-in places (3, 6, 1 arguments).
const APIS: [(u32, u32, u32); 3] = [
    (REG_CREATE_KEY, 0, 3),
    (REG_SET_VALUE, 1, 6),
    (REG_CLOSE_KEY, 2, 1),
];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; push offset phkResult
    let copy = match crate::install::installer::copy(
        0x413DC5,
        0x7D,
        &[0x60, 0x68, 0x5C, 0xBB, 0x51, 0x00],
        &[
            (0x10, REG_CREATE_KEY),
            (0x33, REG_SET_VALUE),
            (0x63, REG_SET_VALUE),
            (0x76, REG_CLOSE_KEY),
        ],
        &[],
        &APIS,
        &[
            (0x17, 0x8010_8420),
            (0x3A, 0x8010_8421),
            (0x6A, 0x8010_8425),
        ],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    // what `prepare` writes: put back at the end
    let _kept = crate::race::viewer::Saved::new(&[
        crate::install::installer::region(STATUS, 1),
        crate::install::installer::region(SOURCE, 0xFF),
        crate::install::installer::region(SOURCE_SIZE, 4),
    ]);
    quiet();
    for source in [0u8, b's'] {
        for fail in [0u32, 1, 2] {
            let prepare = || {
                crate::install::installer::pb(STATUS, 1);
                crate::install::installer::pb(SOURCE, source);
                crate::install::installer::p(SOURCE_SIZE, 4);
                for (n, &(_, place, _)) in APIS.iter().enumerate() {
                    // RegCreateKeyA is place 0, the sets 1, close 2
                    // place 1 is both RegSetValueExA calls (one gives): the second set's
                    // failure (0x80108425) cannot be reached alone, so it is untested.
                    let value = u32::from((fail == 1 && place == 0) || (fail == 2 && place == 1));
                    let _ = n;
                    crate::system::api_stand_ins::set_gives(place, value);
                }
            };
            both(
                &mut t,
                &format!("source {source:#x}, fail {fail}"),
                &copy,
                Regs::SENTINEL,
                &[],
                0,
                &APIS,
                &prepare,
                &|r, _, api| write_registry(r, api),
            );
            for (_, place, _) in APIS {
                crate::system::api_stand_ins::set_gives(place, 0);
            }
        }
    }
    t.finish()
}
