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
use crate::install::installer::{Api, REG_CLOSE_KEY, REG_CREATE_KEY, append_byte, rd, wb};
use crate::install::vars::INSTALLER_NAMES;
use crate::install::vars::INSTALLER_PATHS;
use crate::install::vars::INSTALLER_RUN;
use crate::net::store::NET;
use crate::strings::text_buffer::TEXT_BUFFER;

const HKLM: u32 = 0x8000_0002;

pub fn register_lobby(api: Api) -> Result<(), Refused> {
    use crate::install::set_lobby_value::set_lobby_value;
    if api(
        REG_CREATE_KEY,
        &[
            HKLM,
            crate::install::store::LOBBY_KEYS.registry_key.addr() as u32,
            INSTALLER_RUN.reg_key.addr() as u32,
        ],
    ) != 0
    {
        return Ok(());
    }
    let text = TEXT_BUFFER.text.addr() as u32;
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
        INSTALLER_NAMES.ral_exe.addr() as u32,
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
        INSTALLER_PATHS.installed.addr() as u32,
        api,
    )?;
    set_lobby_value(
        crate::install::store::LOBBY_KEYS
            .name_current_directory
            .addr() as u32,
        INSTALLER_PATHS.installed.addr() as u32,
        api,
    )?;
    api(REG_CLOSE_KEY, &[rd(INSTALLER_RUN.reg_key.addr() as u32)]);
    Ok(())
}
