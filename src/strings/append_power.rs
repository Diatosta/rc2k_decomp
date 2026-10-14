//! `strings::append_power` (0x401CBA): a car's power, bhp or kW.
//!
//! In:  eax = the figure (tenths), ebx = the cursor
//! Out: ebx moved on; everything else preserved
//!
//! See `strings/car_units.rs`. One caller, the car's engine page.
//! Self-test: both unit settings, 506 figures each.

use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::strings::car_units::{append_power, check_unit, dispatch_unit};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("strings::append_power", rva(0x401CBA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_unit(&HOOK, r, call_original, append_power);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if crate::strings::string_table::blob().is_null() {
        return t.skip("string table not loaded");
    }
    check_unit(&mut t, call_original, append_power);
    t.finish()
}
