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
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{
    Api, REG_CLOSE_KEY, REG_CREATE_KEY, REG_KEY, REG_SET_VALUE, SOURCE, SOURCE_SIZE, STATUS, a,
    fatal, g, gb, rd,
};
use crate::race::viewer::regs_entry;

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

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| {
        write_registry(r, &mut crate::install::installer::game_api)
    })
}
