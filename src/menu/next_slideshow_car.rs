//! `menu::next_slideshow_car` (0x469B70): the slideshow's next car.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `dword_755990` + 1, 0 past `dword_75649C` (signed); `dword_755561` and
//! `dword_755994` = 0; `menu::load_slideshow_picture` with eax = 1;
//! `sound::play_car_stream`.
//! Its button in the menu table at 0x74B800.
//! Shadow mode runs the original alone.
//! Self-test: `menu::car_buttons::check_button`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva, shadow};
use crate::menu::car_buttons::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::next_slideshow_car", rva(0x469B70), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub unsafe fn next_slideshow_car(
    call: &mut dyn FnMut(Call) -> Result<(), Refused>,
) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::cars::press`.
    unsafe {
        crate::menu::car_buttons::press(crate::menu::car_buttons::Which::NextSlideshowCar, call)
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
    if let Err(e) = unsafe { next_slideshow_car(&mut |call| call_in_game(call, &entry)) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

// ---- self-test ----

const ORIGINAL_CODE: Original = Original {
    address: 0x469B70,
    len: 0x3E,
    entry: [0x60, 0xA1, 0x90, 0x59, 0x75, 0x00],
    calls: &[0x32, 0x37],
};

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    check_button(&mut t, &ORIGINAL_CODE, &|call| unsafe {
        next_slideshow_car(call)
    });
    t.finish()
}
