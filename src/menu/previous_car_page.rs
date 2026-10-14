//! `menu::previous_car_page` (0x469BAE): the car specification's page before.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `dword_755998` - 1, or 3 from 0 and below (signed); `menu::load_car_background`.
//! The callback at +0x40 of the page's button (menu table at 0x74B688), reached by the
//! arrow keys.
//! Shadow mode runs the original alone.
//! Self-test: `menu::car_buttons::check_button`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::menu::car_buttons::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::previous_car_page", rva(0x469BAE), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub unsafe fn previous_car_page(
    call: &mut dyn FnMut(Call) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        let page: u32 = game_read(PAGE_RVA);
        game_write(PAGE_RVA, if (page as i32) > 0 { page - 1 } else { 3 });
        call(Call::Background)
    }
}

// ---- hook ----

extern "C" fn dispatch(r: &mut Regs) {
    HOOK.count_call();
    if shadow() {
        unsafe { call_original(r) };
        return;
    }
    let entry = *r;
    if let Err(e) = unsafe { previous_car_page(&mut |call| call_in_game(call, &entry)) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

// ---- self-test ----

const ORIGINAL_CODE: Original = Original {
    address: 0x469BAE,
    len: 0x23,
    entry: [0x60, 0x83, 0x3D, 0x98, 0x59, 0x75],
    calls: &[0x1C],
};

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    check_button(&mut t, &ORIGINAL_CODE, &|call| unsafe {
        previous_car_page(call)
    });
    t.finish()
}
