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
use crate::install::store::{LOBBY_KEYS, text_of};
use crate::net::store::NET;
use crate::strings::text_buffer::TEXT_BUFFER;

pub fn register_lobby(
    g: &'static crate::install::owner::Install,
    registry: &mut dyn Registry,
) -> Result<(), Refused> {
    use crate::install::set_lobby_value::set_lobby_value;
    let Ok(key) = registry.create(text_of(&LOBBY_KEYS.registry_key)) else {
        return Ok(());
    };
    g.run.reg_key.set(key);
    // Converted: `rc2k_core::install::lobby::guid_text`.
    let guid = rc2k_core::install::lobby::guid_text(&NET.app_guid.read());
    let mut text = guid.to_vec();
    text.push(0);
    TEXT_BUFFER.text.write_at(0, &text, "the text buffer")?;
    set_lobby_value(g, text_of(&LOBBY_KEYS.name_guid), &guid, registry)?;
    set_lobby_value(
        g,
        text_of(&LOBBY_KEYS.name_file),
        &g.names.ral_exe.text(),
        registry,
    )?;
    TEXT_BUFFER.text.set(0, 0);
    set_lobby_value(g, text_of(&LOBBY_KEYS.name_command_line), b"", registry)?;
    let installed = g.paths.installed.text();
    set_lobby_value(g, text_of(&LOBBY_KEYS.name_path), &installed, registry)?;
    set_lobby_value(
        g,
        text_of(&LOBBY_KEYS.name_current_directory),
        &installed,
        registry,
    )?;
    registry.close(g.run.reg_key.get());
    Ok(())
}
