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
//! (`sub_40171A`) done here. Shadow mode runs the original alone. Self-test: a copy whose
//! create, close and five value-sets go to stand-ins, against the Rust version (the GUID
//! built in `text`, which is compared).
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::framework::Regs;
use crate::install::installer::{
    Api, INSTALLED, Part, REG_CLOSE_KEY, REG_CREATE_KEY, REG_KEY, TEXT, a, append_byte, rb, rd, wb,
};
use crate::race::viewer::regs_entry;

const KEY_NAME: usize = 0x5AB419; // "SOFTWARE\Microsoft\DirectPlay\Applications\RC"
const GUID: usize = 0x5AB300; // dword_5AB300
const SET_LOBBY: usize = 0x4190D0; // install::set_lobby_value
const NAME_GUID: usize = 0x5AB463; // "Guid"
const NAME_FILE: usize = 0x5AB468; // "File"
const RAL_EXE: usize = 0x51BE48; // "ral.exe"
const NAME_CMDLINE: usize = 0x5AB46D; // "CommandLine"
const NAME_PATH: usize = 0x5AB479; // "Path"
const NAME_CURDIR: usize = 0x5AB47E; // "CurrentDirectory"
const HKLM: u32 = 0x8000_0002;

pub fn register_lobby(entry: &Regs, part: Part, api: Api) -> Result<Regs, Refused> {
    let mut r = *entry;
    if api(REG_CREATE_KEY, &[HKLM, a(KEY_NAME), a(REG_KEY)]) != 0 {
        return Ok(r);
    }
    let text = a(TEXT);
    // Converted: `rc2k_core::install::lobby::guid_text`.
    let guid: [u8; 16] = std::array::from_fn(|n| rb(a(GUID) + n as u32));
    let guid = rc2k_core::install::lobby::guid_text(&guid);
    for (n, &b) in guid.iter().enumerate() {
        wb(text + n as u32, b);
    }
    r.ebx = text + guid.len() as u32;
    append_byte(&mut r, 0);
    let set = |name: usize, data: u32, r: &mut Regs, part: Part| -> Result<(), Refused> {
        r.eax = a(name);
        r.ebx = data;
        part(crate::install::installer::at(SET_LOBBY), r)
    };
    set(NAME_GUID, text, &mut r, part)?;
    set(NAME_FILE, a(RAL_EXE), &mut r, part)?;
    wb(text, 0);
    set(NAME_CMDLINE, text, &mut r, part)?;
    set(NAME_PATH, a(INSTALLED), &mut r, part)?;
    set(NAME_CURDIR, a(INSTALLED), &mut r, part)?;
    api(REG_CLOSE_KEY, &[rd(a(REG_KEY))]);
    Ok(Regs { ..*entry })
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| {
        register_lobby(
            r,
            &mut crate::install::installer::game_part,
            &mut crate::install::installer::game_api,
        )
    })
}
