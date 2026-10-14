//! `menu::previous_car` (0x469A6F): the car specification's car before.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `dword_755990` - 1, or `dword_75649C` from 0; `dword_755561` = 0;
//! `menu::load_car_background`; `sound::play_car_stream`.
//! The callback at +0x40 of the car's button (menu table at 0x74B62C), reached by the
//! arrow keys.
//! Shadow mode runs the original alone.
//! Self-test: `menu::car_buttons::check_button`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::menu::car_buttons::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::previous_car", rva(0x469A6F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub unsafe fn previous_car(
    call: &mut dyn FnMut(Call) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        let car: u32 = game_read(CAR_RVA);
        let car = if car == 0 {
            game_read(LAST_CAR_RVA)
        } else {
            car.wrapping_sub(1)
        };
        game_write(CAR_RVA, car);
        game_write(LEAVE_RVA, 0u32);
        call(Call::Background)?;
        call(Call::Stream)
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
    if let Err(e) = unsafe { previous_car(&mut |call| call_in_game(call, &entry)) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

// ---- self-test ----

const ORIGINAL_CODE: Original = Original {
    address: 0x469A6F,
    len: 0x32,
    entry: [0x60, 0x83, 0x3D, 0x90, 0x59, 0x75],
    calls: &[0x26, 0x2B],
};

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    check_button(&mut t, &ORIGINAL_CODE, &|call| unsafe {
        previous_car(call)
    });
    t.finish()
}
