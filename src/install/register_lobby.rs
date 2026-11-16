//! The game registered as a DirectPlay lobby program under
//! `HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\DirectPlay\Applications\RC`: five string values (`Guid`,
//! `File`, `CommandLine`, `Path`, `CurrentDirectory`) set with `install::set_lobby_value`.

use crate::errors::Refused;
use crate::install::read_registry::Registry;
use crate::install::store::text_of;

pub fn register_lobby(
    install: &crate::install::owner::Install,
    registry: &mut dyn Registry,
    strings: &crate::strings::owner::Strings,
) -> Result<(), Refused> {
    use crate::install::set_lobby_value::set_lobby_value;
    let Ok(key) = registry.create(text_of(&install.lobby_keys.registry_key)) else {
        return Ok(());
    };
    install.run.reg_key.set(key);
    let guid = rc2k_core::install::lobby::guid_text(&crate::net::store::APP_GUID);
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
