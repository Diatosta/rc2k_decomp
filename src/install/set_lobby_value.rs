//! `install::set_lobby_value` (0x4190D0): one string value set under the lobby's registry
//! key (`register_lobby`): its data's length found (`strings::text_length`) and one more
//! (the NUL), and `RegSetValueExA(phkResult, name, 0, REG_SZ, data, len + 1)`.
//!
//! In:  eax the value's name, ebx the data (NUL-terminated)
//! Out: eax `RegSetValueExA`'s result; ebx, edx kept, ecx the length plus one
//! Caller `install::register_lobby` (five times). The call through `api`
//! (`installer::game_api`: `RegSetValueExA`). Shadow mode runs the original alone.
//! Self-test: a copy whose two calls go to stand-ins, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{Api, REG_KEY, REG_SET_VALUE, a, both, quiet, rd};
use crate::race::viewer::{dispatch_parts, regs_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::set_lobby_value", rva(0x4190D0), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const REG_SZ: u32 = 1;

pub fn set_lobby_value(entry: &Regs, api: Api) -> Result<Regs, Refused> {
    let mut r = *entry;
    let len = unsafe { crate::strings::text_length::text_length(r.ebx as *const u8) }?;
    r.ecx = len.wrapping_add(1);
    let key = rd(a(REG_KEY));
    r.eax = api(REG_SET_VALUE, &[key, r.eax, 0, REG_SZ, r.ebx, r.ecx]);
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        set_lobby_value(r, &mut crate::install::installer::game_api)
    });
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| {
        set_lobby_value(r, &mut crate::install::installer::game_api)
    })
}

const APIS: [(u32, u32, u32); 1] = [(REG_SET_VALUE, 1, 6)];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // call strings__text_length; inc ecx
    let copy = match crate::install::installer::copy(
        0x4190D0,
        0x19,
        &[0xE8, 0xE3, 0x91, 0xFE, 0xFF],
        &[(0x0, 0x4022B8), (0x13, REG_SET_VALUE)],
        &[],
        &APIS,
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    quiet();
    crate::system::api_stand_ins::set_gives(1, 0);
    let mut name = *b"Guid\0";
    let mut data = *b"{0123-abc}\0";
    for text in [&b"ral.exe\0"[..], b"\0", b"a longer value than usual\0"] {
        let mut d = text.to_vec();
        let prepare = || {};
        both(
            &mut t,
            &format!("value {}", crate::strings::utils::bytes_str(text)),
            &copy,
            Regs {
                eax: name.as_mut_ptr() as u32,
                ebx: d.as_mut_ptr() as u32,
                ..Regs::SENTINEL
            },
            &[],
            0,
            &APIS,
            &prepare,
            &|r, _, api| set_lobby_value(r, api),
        );
        let _ = &mut data;
    }
    t.finish()
}
