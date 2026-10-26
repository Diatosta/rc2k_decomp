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
//! `Win32_System_Registry`); the standalone makes them too .
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{Api, REG_CLOSE_KEY, REG_CREATE_KEY, REG_SET_VALUE, fatal, rd};
use crate::install::vars::INSTALLER_NAMES;
use crate::install::vars::INSTALLER_RUN;

const HKLM: u32 = 0x8000_0002;
const REG_SZ: u32 = 1;
const REG_BINARY: u32 = 3;

pub fn write_registry(api: Api) -> Result<(), Refused> {
    let key_at = INSTALLER_RUN.reg_key.addr();
    if api(
        REG_CREATE_KEY,
        &[HKLM, INSTALLER_NAMES.key_name.addr(), key_at],
    ) != 0
    {
        return Err(fatal(0x8010_8420));
    }
    let key = rd(key_at);
    if api(
        REG_SET_VALUE,
        &[
            key,
            INSTALLER_NAMES.status_name.addr(),
            0,
            REG_BINARY,
            INSTALLER_NAMES.status.addr(),
            1,
        ],
    ) != 0
    {
        return Err(fatal(0x8010_8421));
    }
    if INSTALLER_NAMES.source.get(0) != 0
        && api(
            REG_SET_VALUE,
            &[
                key,
                INSTALLER_NAMES.source_name.addr(),
                0,
                REG_SZ,
                INSTALLER_NAMES.source.addr(),
                INSTALLER_NAMES.source_size.get(),
            ],
        ) != 0
    {
        return Err(fatal(0x8010_8425));
    }
    api(REG_CLOSE_KEY, &[key]);
    Ok(())
}
