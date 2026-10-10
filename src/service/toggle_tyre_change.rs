//! `service::toggle_tyre_change` (0x46C0AE): flips the flag of the highlighted menu item
//! and, if the flag is then not 1, puts the chosen tyre (`dword_75AC9C`)
//! back to the installed value (`dword_75ACA0`).
//!
//! In:  nothing
//! Out: every register preserved
//! Refuses a highlighted item past the flag table. "Toggle change" is a reading of that.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::service::setup_values::{Change, check_change, dispatch_change};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "service::toggle_tyre_change",
    rva(0x46C0AE),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version: `setup_values::toggle_change` with this ----

pub const CHANGE: Change = Change {
    chosen_rva: rva(0x75AC9C),
    installed_rva: rva(0x75ACA0),
};

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_change(&HOOK, r, call_original, CHANGE);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    check_change(&mut t, call_original, CHANGE);
    t.finish()
}
