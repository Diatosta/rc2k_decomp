//! `install::register_lobby` (0x419030): the game registered as a DirectPlay lobby program,
//! under `HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\DirectPlay\Applications\RC`: the key opened
//! (`RegCreateKeyA`; failing: nothing more), then five string values set
//! (`install::set_lobby_value`): `Guid` the application's GUID (`app_guid`) as
//! `{XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}` built in `text`, `File` "ral.exe", `CommandLine`
//! empty, `Path` and `CurrentDirectory` the installed folder (`installed`); the key
//! closed (`RegCloseKey`).
//!
//! Caller `install::install_files`. The registry calls through `api`; the GUID formatting
//! (0x40171A) done here.

use crate::errors::Refused;
use crate::install::read_registry::Registry;
use crate::install::store::text_of;

pub fn register_lobby(
    install: &'static crate::install::owner::Install,
    registry: &mut dyn Registry,
    strings: &crate::strings::owner::Strings,
    net: &crate::net::owner::Net,
) -> Result<(), Refused> {
    use crate::install::set_lobby_value::set_lobby_value;
    let Ok(key) = registry.create(text_of(&install.lobby_keys.registry_key)) else {
        return Ok(());
    };
    install.run.reg_key.set(key);
    // Converted: `rc2k_core::install::lobby::guid_text`.
    let guid = rc2k_core::install::lobby::guid_text(&net.vars.app_guid.read());
    let mut text = guid.to_vec();
    text.push(0);
    strings
        .text
        .buffer
        .text
        .write_at(0, &text, "the text buffer")?;
    set_lobby_value(
        install,
        text_of(&install.lobby_keys.name_guid),
        &guid,
        registry,
    )?;
    set_lobby_value(
        install,
        text_of(&install.lobby_keys.name_file),
        &install.names.ral_exe.text(),
        registry,
    )?;
    strings.text.buffer.text.set(0, 0);
    set_lobby_value(
        install,
        text_of(&install.lobby_keys.name_command_line),
        b"",
        registry,
    )?;
    let installed = install.paths.installed.text();
    set_lobby_value(
        install,
        text_of(&install.lobby_keys.name_path),
        &installed,
        registry,
    )?;
    set_lobby_value(
        install,
        text_of(&install.lobby_keys.name_current_directory),
        &installed,
        registry,
    )?;
    registry.close(install.run.reg_key.get());
    Ok(())
}
