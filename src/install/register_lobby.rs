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
use crate::install::read_registry::Registry;
use crate::install::store::LOBBY_KEYS;
use crate::install::vars::{INSTALLER_NAMES, INSTALLER_PATHS, INSTALLER_RUN};
use crate::net::store::NET;
use crate::strings::text_buffer::TEXT_BUFFER;

pub fn register_lobby(registry: &mut dyn Registry) -> Result<(), Refused> {
    use crate::install::set_lobby_value::set_lobby_value;
    let Ok(key) = registry.create(&LOBBY_KEYS.registry_key.text()) else {
        return Ok(());
    };
    INSTALLER_RUN.reg_key.set(key);
    // Converted: `rc2k_core::install::lobby::guid_text`.
    let guid = rc2k_core::install::lobby::guid_text(&NET.app_guid.read());
    let mut text = guid.to_vec();
    text.push(0);
    TEXT_BUFFER.text.write_at(0, &text, "the text buffer")?;
    set_lobby_value(&LOBBY_KEYS.name_guid.text(), &guid, registry)?;
    set_lobby_value(
        &LOBBY_KEYS.name_file.text(),
        &INSTALLER_NAMES.ral_exe.text(),
        registry,
    )?;
    TEXT_BUFFER.text.set(0, 0);
    set_lobby_value(&LOBBY_KEYS.name_command_line.text(), b"", registry)?;
    let installed = INSTALLER_PATHS.installed.text();
    set_lobby_value(&LOBBY_KEYS.name_path.text(), &installed, registry)?;
    set_lobby_value(
        &LOBBY_KEYS.name_current_directory.text(),
        &installed,
        registry,
    )?;
    registry.close(INSTALLER_RUN.reg_key.get());
    Ok(())
}
