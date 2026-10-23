//! `install::unregister_lobby` (0x4190E9): the game's DirectPlay lobby key removed:
//! `RegDeleteKeyA(HKEY_LOCAL_MACHINE, "SOFTWARE\Microsoft\DirectPlay\Applications\RC")`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` / `popa`)
//! Caller `install::uninstall`. The call through `api`. Shadow mode runs the original
//! alone. Self-test: a copy whose one call goes to a stand-in, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{Api, REG_DELETE_KEY, a, both, quiet};
use crate::race::viewer::{dispatch_parts, regs_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::unregister_lobby", rva(0x4190E9), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const KEY_NAME: usize = 0x5AB419; // "SOFTWARE\Microsoft\DirectPlay\Applications\RC"
const HKLM: u32 = 0x8000_0002;

pub fn unregister_lobby(entry: &Regs, api: Api) -> Result<Regs, Refused> {
    api(REG_DELETE_KEY, &[HKLM, a(KEY_NAME)]);
    Ok(*entry)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_parts(&HOOK, r, call_original, &|r| {
        unregister_lobby(r, &mut crate::install::installer::game_api)
    });
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| {
        unregister_lobby(r, &mut crate::install::installer::game_api)
    })
}

const APIS: [(u32, u32, u32); 1] = [(REG_DELETE_KEY, 0, 2)];

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    // pusha; push offset aSoftwareMicros
    let copy = match crate::install::installer::copy(
        0x4190E9,
        0x12,
        &[0x60, 0x68, 0x19, 0xB4, 0x5A, 0x00],
        &[(0xB, REG_DELETE_KEY)],
        &[],
        &APIS,
        &[],
        &[],
    ) {
        Ok(c) => c,
        Err(why) => return t.skip(&why),
    };
    quiet();
    crate::system::api_stand_ins::set_gives(0, 0);
    both(
        &mut t,
        "delete the lobby key",
        &copy,
        Regs::SENTINEL,
        &[],
        0,
        &APIS,
        &|| {},
        &|r, _, api| unregister_lobby(r, api),
    );
    t.finish()
}
