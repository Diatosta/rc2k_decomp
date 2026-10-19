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

/// `sub_40171A`: the 16-byte GUID at `at` formatted at `cursor` as the registry wants (no
/// braces); the cursor past it.
fn format_guid(at: u32, mut cursor: u32) -> u32 {
    let hex = |value: u32, digits: u32, cursor: &mut u32| {
        let bytes = crate::strings::hex_append::hex_append(value, digits).unwrap_or_default();
        for b in bytes {
            wb(*cursor, b);
            *cursor += 1;
        }
    };
    let dash = |cursor: &mut u32| {
        wb(*cursor, b'-');
        *cursor += 1;
    };
    hex(rd(at), 8, &mut cursor);
    dash(&mut cursor);
    hex(rb(at + 4) as u32 | (rb(at + 5) as u32) << 8, 4, &mut cursor);
    dash(&mut cursor);
    hex(rb(at + 6) as u32 | (rb(at + 7) as u32) << 8, 4, &mut cursor);
    dash(&mut cursor);
    // ror ax, 8 of the word at +8: the two bytes swapped
    hex(rb(at + 9) as u32 | (rb(at + 8) as u32) << 8, 4, &mut cursor);
    dash(&mut cursor);
    for n in 0xA..0x10u32 {
        hex(rb(at + n) as u32, 2, &mut cursor);
    }
    cursor
}

pub fn register_lobby(entry: &Regs, part: Part, api: Api) -> Result<Regs, Refused> {
    let mut r = *entry;
    if api(REG_CREATE_KEY, &[HKLM, a(KEY_NAME), a(REG_KEY)]) != 0 {
        return Ok(r);
    }
    let text = a(TEXT);
    wb(text, b'{');
    let cursor = format_guid(a(GUID), text + 1);
    wb(cursor, b'}');
    r.ebx = cursor + 1;
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
