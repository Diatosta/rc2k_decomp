//! `net::release_dplay` (0x41864E): with a DirectPlay object (`g_dplay` not 0), "NET:
//! Destroy dplay" logged (level 2, a line), the object released (`IDirectPlay4A` +8) and
//! `g_dplay` made 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`); without an object the flags of
//!      `cmp g_dplay, 0`, else those `Release` left (not modelled: the Rust version leaves
//!      the compare's; the one caller does not read them)
//! Caller: `net::destroy_connection`. The log through `part`, `Release` from Rust
//! (`net::dplay`, section 15). Refuses nothing. Shadow mode runs the original alone.
//! Self-test: a copy whose variable is the test's, whose log goes to a noting stand-in and
//! whose DirectPlay is a test object noting its arguments, against the Rust version.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::net::dplay::{RELEASE, method};
use crate::net::lobby::{FLAGS, TestObject, cmp_flags, log_text, run_both};
use crate::net::message_vars::{MessageVars, TestVars, as_rust, get, set};
use crate::net::{LOG_TEXT_2_NL, Part, a, dispatch_net, net_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("net::release_dplay", rva(0x41864E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const TEXT: usize = 0x5C2E4C; // "NET: Destroy dplay"

/// The Rust version on the variables at `v`.
pub fn release_dplay_with(r: &mut Regs, v: &MessageVars, part: Part) -> Result<(), Refused> {
    let object = get(v.dplay);
    cmp_flags(r, object, 0);
    if object != 0 {
        log_text(LOG_TEXT_2_NL, a(TEXT), r, part)?;
        unsafe { method(get(v.dplay), RELEASE, &[]) };
        set(v.dplay, 0);
    }
    Ok(())
}

/// The Rust version.
pub fn release_dplay(r: &mut Regs, part: Part) -> Result<(), Refused> {
    release_dplay_with(r, &MessageVars::game(), part)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_net(&HOOK, r, call_original, release_dplay);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    net_entry(r, release_dplay)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let tv = TestVars::new(0);
    // pusha; cmp g_dplay, 0
    let entry = [0x60, 0x83, 0x3D, 0x88, 0xB2, 0x5A, 0x00, 0x00];
    let copy = match tv.copy(0x41864E, 0x2E, &entry, &[0x11], &[LOG_TEXT_2_NL], 3) {
        Ok(c) => c,
        Err(why) => return t.skip(why),
    };
    let v = tv.v;
    let dplay = TestObject::dplay();
    let dp = dplay.addr();
    for object in [0u32, dp] {
        run_both(
            &mut t,
            &format!("object {object:#x}"),
            copy.addr(),
            Regs::SENTINEL,
            &[tv.region()],
            if object != 0 { 0 } else { FLAGS },
            &[],
            &|| set(v.dplay, object),
            &as_rust(|r, part| release_dplay_with(r, &v, part)),
        );
    }
    drop(dplay);
    t.finish()
}
