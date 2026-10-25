//! `install::register_lobby` (0x419030): the game registered as a DirectPlay lobby program,
//! under `HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\DirectPlay\Applications\RC`: the key opened
//! (`RegCreateKeyA`; failing: nothing more), then five string values set
//! (`install::set_lobby_value`): `Guid` the application's GUID (`dword_5AB300`) as
//! `{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}` built in `text`, `File` "ral.exe", `CommandLine`
//! empty, `Path` and `CurrentDirectory` the installed folder (`byte_95E7F8`); the key
//! closed (`RegCloseKey`).
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//! Caller `install::install_files`. The registry calls through `api`; the GUID formatting
//! (`sub_40171A`) done here.
//!
//! Conversion phase 3 (2026-10-23): plain Rust with typed arguments; the registers above
//! are the original's.

use crate::errors::Refused;
use crate::install::installer::{
    Api, INSTALLED, REG_CLOSE_KEY, REG_CREATE_KEY, REG_KEY, TEXT, a, append_byte, rd, wb,
};
use crate::net::store::NET;

const RAL_EXE: usize = 0x51BE48; // "ral.exe"
const HKLM: u32 = 0x8000_0002;

pub fn register_lobby(api: Api) -> Result<(), Refused> {
    use crate::install::set_lobby_value::set_lobby_value;
    if api(
        REG_CREATE_KEY,
        &[
            HKLM,
            crate::install::store::LOBBY_KEYS.registry_key.addr() as u32,
            a(REG_KEY),
        ],
    ) != 0
    {
        return Ok(());
    }
    let text = a(TEXT);
    // Converted: `rc2k_core::install::lobby::guid_text`.
    let guid: [u8; 16] = std::array::from_fn(|n| NET.app_guid.get(n));
    let guid = rc2k_core::install::lobby::guid_text(&guid);
    for (n, &b) in guid.iter().enumerate() {
        wb(text + n as u32, b);
    }
    append_byte(text + guid.len() as u32, 0);
    set_lobby_value(
        crate::install::store::LOBBY_KEYS.name_guid.addr() as u32,
        text,
        api,
    )?;
    set_lobby_value(
        crate::install::store::LOBBY_KEYS.name_file.addr() as u32,
        a(RAL_EXE),
        api,
    )?;
    wb(text, 0);
    set_lobby_value(
        crate::install::store::LOBBY_KEYS.name_command_line.addr() as u32,
        text,
        api,
    )?;
    set_lobby_value(
        crate::install::store::LOBBY_KEYS.name_path.addr() as u32,
        a(INSTALLED),
        api,
    )?;
    set_lobby_value(
        crate::install::store::LOBBY_KEYS
            .name_current_directory
            .addr() as u32,
        a(INSTALLED),
        api,
    )?;
    api(REG_CLOSE_KEY, &[rd(a(REG_KEY))]);
    Ok(())
}
