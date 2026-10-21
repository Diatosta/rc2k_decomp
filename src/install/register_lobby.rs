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

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{
    Api, INSTALLED, Part, REG_CLOSE_KEY, REG_CREATE_KEY, REG_KEY, TEXT, a, append_byte, both, fill,
    quiet, rb, rd, region, wb,
};
use crate::race::viewer::{Saved, dispatch_parts};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::register_lobby", rva(0x419030), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

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

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        register_lobby(
            r,
            &mut crate::install::installer::game_part,
            &mut crate::install::installer::game_api,
        )
    });
}

const APIS: [(u32, u32, u32); 2] = [(REG_CREATE_KEY, 0, 3), (REG_CLOSE_KEY, 2, 1)];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; push offset phkResult
    let copy = match crate::install::installer::copy(
        0x419030,
        0xA0,
        &[0x60, 0x68, 0x5C, 0xBB, 0x51, 0x00],
        &[
            (0x10, REG_CREATE_KEY),
            (0x2D, 0x40171A),
            (0x36, 0x401F16),
            (0x47, SET_LOBBY as u32),
            (0x58, SET_LOBBY as u32),
            (0x6C, SET_LOBBY as u32),
            (0x7D, SET_LOBBY as u32),
            (0x8E, SET_LOBBY as u32),
            (0x99, REG_CLOSE_KEY),
        ],
        &[SET_LOBBY as u32],
        &APIS,
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    quiet();
    let regions = [region(TEXT, 0x40), region(GUID, 0x10)];
    let saved = Saved::new(&regions);
    for create in [0u32, 1] {
        let prepare = || {
            crate::system::api_stand_ins::set_gives(0, create);
            fill(a(TEXT), 0x40, 0x11);
            fill(a(GUID), 0x10, 0x5C);
        };
        both(
            &mut t,
            &format!("create gives {create}"),
            &copy,
            Regs::SENTINEL,
            &regions,
            0,
            &APIS,
            &prepare,
            &|r, part, api| register_lobby(r, part, api),
        );
    }
    crate::system::api_stand_ins::set_gives(0, 0);
    let _ = rd;
    drop(saved);
    t.finish()
}
