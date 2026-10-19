//! `net::destroy_player` (0x41916B): DirectPlay's `DestroyPlayer` (`IDirectPlay4A` +0x24)
//! on `g_netPlayerId` through `g_dplay`, and the id made 0 — but only when it is 0 already
//! (`cmp g_netPlayerId, 0; jnz` past the call: the game's slip, section 16; this machine's
//! player is never destroyed, `DestroyPlayer(0)` is called instead).
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); the flags of `cmp g_netPlayerId, 0` when
//!      it is not 0, else those the method left (not modelled: the Rust version leaves the
//!      compare's; the one caller does not read them)
//! Caller: `net::destroy_connection`. `DestroyPlayer` from Rust (`net::dplay`, section 15).
//! Refuses nothing. Shadow mode runs the original alone. Self-test: a copy whose two
//! variables are the test's and whose DirectPlay is a test object noting its arguments,
//! against the Rust version: id 0, id not 0.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::dplay::{DESTROY_PLAYER, method};
use crate::net::lobby::{FLAGS, TestObject, cmp_flags, run_both};
use crate::net::message_vars::{MessageVars, TestVars, as_rust, get, set};
use crate::net::{Part, dispatch_net};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::destroy_player", rva(0x41916B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version on the variables at `v`.
pub fn destroy_player_with(r: &mut Regs, v: &MessageVars) {
    let id = get(v.player_id);
    cmp_flags(r, id, 0);
    if id == 0 {
        unsafe { method(get(v.dplay), DESTROY_PLAYER, &[id]) };
        set(v.player_id, 0);
    }
}

/// The Rust version.
pub fn destroy_player(r: &mut Regs, _part: Part) -> Result<(), Refused> {
    destroy_player_with(r, &MessageVars::game());
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_net(&HOOK, r, call_original, destroy_player);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let tv = TestVars::new(0);
    // pusha; cmp g_netPlayerId, 0
    let entry = [0x60, 0x83, 0x3D, 0x70, 0xB2, 0x5A, 0x00, 0x00];
    let copy = match tv.copy(0x41916B, 0x27, &entry, &[], &[], 4) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let v = tv.v;
    let dplay = TestObject::dplay();
    let dp = dplay.addr();
    for id in [0u32, 0x0000_6001, 0x8000_0002] {
        run_both(
            &mut t,
            &format!("player {id:#x}"),
            copy.addr(),
            Regs::SENTINEL,
            &[tv.region()],
            // After the method its flags (the test object's code's).
            if id == 0 { 0 } else { FLAGS },
            &[],
            &|| {
                set(v.dplay, dp);
                set(v.player_id, id);
            },
            &as_rust(|r, _| {
                destroy_player_with(r, &v);
                Ok(())
            }),
        );
    }
    drop(dplay);
    t.finish()
}
