//! `net::release_lobby` (0x41900F): with a lobby object (`g_dplayLobby` not 0), the object
//! released (`IDirectPlayLobby2A` +8) and `g_dplayLobby` made 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); without an object the flags of
//!      `cmp g_dplayLobby, 0`, else those `Release` left (not modelled: the Rust version
//!      leaves the compare's; the one caller does not read them)
//! Caller: `net::destroy_connection`. `Release` from Rust (`net::dplay`, section 15).
//! Refuses nothing. Shadow mode runs the original alone. Self-test: a copy whose variable
//! is the test's and whose lobby is a test object noting its arguments, against the Rust
//! version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::dplay::{RELEASE, method};
use crate::net::lobby::{FLAGS, TestObject, cmp_flags, run_both};
use crate::net::message_vars::{MessageVars, TestVars, as_rust, get, set};
use crate::net::{Part, dispatch_net};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::release_lobby", rva(0x41900F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version on the variables at `v`.
pub fn release_lobby_with(r: &mut Regs, v: &MessageVars) {
    let object = get(v.lobby);
    cmp_flags(r, object, 0);
    if object != 0 {
        unsafe { method(get(v.lobby), RELEASE, &[]) };
        set(v.lobby, 0);
    }
}

/// The Rust version.
pub fn release_lobby(r: &mut Regs, _part: Part) -> Result<(), Refused> {
    release_lobby_with(r, &MessageVars::game());
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_net(&HOOK, r, call_original, release_lobby);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let tv = TestVars::new(0);
    // pusha; cmp g_dplayLobby, 0
    let entry = [0x60, 0x83, 0x3D, 0x8C, 0xB2, 0x5A, 0x00, 0x00];
    let copy = match tv.copy(0x41900F, 0x21, &entry, &[], &[], 3) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let v = tv.v;
    let lobby = TestObject::lobby();
    let object_at = lobby.addr();
    for object in [0u32, object_at] {
        run_both(
            &mut t,
            &format!("object {object:#x}"),
            copy.addr(),
            Regs::SENTINEL,
            &[tv.region()],
            if object != 0 { 0 } else { FLAGS },
            &[],
            &|| set(v.lobby, object),
            &as_rust(|r, _| {
                release_lobby_with(r, &v);
                Ok(())
            }),
        );
    }
    drop(lobby);
    t.finish()
}
