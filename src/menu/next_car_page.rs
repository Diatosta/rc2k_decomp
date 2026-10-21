//! `menu::next_car_page` (0x469BD1): the car specification's next page.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `dword_755998` + 1, from 3 back to 0; `menu::load_car_background`.
//! Its button in the menu table at 0x74B688.
//! Shadow mode runs the original alone.
//! Self-test: `menu::car_buttons::check_button`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::menu::car_buttons::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::next_car_page", rva(0x469BD1), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub unsafe fn next_car_page(
    call: &mut dyn FnMut(Call) -> Result<(), Refused>,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::cars::press`.
    unsafe { crate::menu::car_buttons::press(crate::menu::car_buttons::Which::NextPage, call) }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let entry = *r;
    if let Err(e) = unsafe { next_car_page(&mut |call| call_in_game(call, &entry)) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

// ---- self-test ----

const ORIGINAL_CODE: Original = Original {
    address: 0x469BD1,
    len: 0x23,
    entry: [0x60, 0x83, 0x3D, 0x98, 0x59, 0x75],
    calls: &[0x1C],
};

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    check_button(&mut t, &ORIGINAL_CODE, &|call| unsafe {
        next_car_page(call)
    });
    t.finish()
}
