//! `install::write_registry` (0x413DC5): the installation written to the registry, under
//! `HKEY_LOCAL_MACHINE\SOFTWARE\Magnetic Fields\RC99`: the value `status` (one byte,
//! `g_installStatus`, type binary) and, if `g_installSource` is not empty, `source`
//! (`source_size` bytes, type string). The key opened (`RegCreateKeyA`) and closed
//! (`RegCloseKey`).
//!
//! A failed create, or a failed set of either value, is fatal error 0x80108420, 0x80108421
//! or 0x80108425. Three callers (`install::install_files`, `uninstall`, and
//! `install::run_installer`). The registry calls through `api` (`installer::game_api` in the game:
//! `windows-sys`
//! `Win32_System_Registry`); the standalone makes them too .

use crate::errors::Refused;
use crate::install::installer::fatal;
use crate::install::read_registry::Registry;

const REG_SZ: u32 = 1;
const REG_BINARY: u32 = 3;

pub fn write_registry(
    install: &crate::install::owner::Install,
    registry: &mut dyn Registry,
) -> Result<(), Refused> {
    let names = &install.names;
    let Ok(key) = registry.create(&names.key_name.text()) else {
        return Err(fatal(0x8010_8420));
    };
    install.run.reg_key.set(key);
    if registry.set_value(
        key,
        &names.status_name.text(),
        REG_BINARY,
        &[names.status.get()],
    ) != 0
    {
        return Err(fatal(0x8010_8421));
    }
    if names.source.get(0) != 0 {
        let size = names.source_size.get() as usize;
        let source = names.source.read();
        let data = source
            .get(..size)
            .ok_or(crate::errors::BadInput("a source size past its buffer"))?;
        if registry.set_value(key, &names.source_name.text(), REG_SZ, data) != 0 {
            return Err(fatal(0x8010_8425));
        }
    }
    registry.close(key);
    Ok(())
}
