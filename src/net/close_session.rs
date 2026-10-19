//! `net::close_session` (0x418DD3): with a session open (`dword_52ACFC` 1), the switch made
//! 0 and DirectPlay's `Close` (`IDirectPlay4A` +0x10) called through `g_dplay`.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); the flags of `cmp dword_52ACFC, 1` when it
//!      is not 1, else those the method left (not modelled: the Rust version leaves the
//!      compare's; the one caller does not read them)
//! Caller: `net::destroy_connection`. `Close` from Rust (`net::dplay`, section 15). Refuses
//! nothing. Shadow mode runs the original alone. Self-test: a copy whose two variables are
//! the test's and whose DirectPlay is a test object noting its arguments, against the Rust
//! version: open, not open, 2.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::dplay::{CLOSE, method};
use crate::net::lobby::{FLAGS, TestObject, cmp_flags, run_both};
use crate::net::message_vars::{MessageVars, TestVars, as_rust, get, set};
use crate::net::{Part, dispatch_net};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::close_session", rva(0x418DD3), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version on the variables at `v`.
pub fn close_session_with(r: &mut Regs, v: &MessageVars) {
    let open = get(v.session_open);
    cmp_flags(r, open, 1);
    if open == 1 {
        set(v.session_open, 0);
        unsafe { method(get(v.dplay), CLOSE, &[]) };
    }
}

/// The Rust version.
pub fn close_session(r: &mut Regs, _part: Part) -> Result<(), Refused> {
    close_session_with(r, &MessageVars::game());
    Ok(())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_net(&HOOK, r, call_original, close_session);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let tv = TestVars::new(0);
    // pusha; cmp dword_52ACFC, 1
    let entry = [0x60, 0x83, 0x3D, 0xFC, 0xAC, 0x52, 0x00, 0x01];
    let copy = match tv.copy(0x418DD3, 0x21, &entry, &[], &[], 3) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let v = tv.v;
    let dplay = TestObject::dplay();
    let dp = dplay.addr();
    for open in [0u32, 1, 2] {
        run_both(
            &mut t,
            &format!("open {open}"),
            copy.addr(),
            Regs::SENTINEL,
            &[tv.region()],
            if open == 1 { 0 } else { FLAGS },
            &[],
            &|| {
                set(v.dplay, dp);
                set(v.session_open, open);
            },
            &as_rust(|r, _| {
                close_session_with(r, &v);
                Ok(())
            }),
        );
    }
    drop(dplay);
    t.finish()
}
