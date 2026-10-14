//! `menu::previous_slideshow_car` (0x469B35): the slideshow's car before.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `dword_755990` - 1, `dword_75649C` below 0 (signed); `dword_755561` and
//! `dword_755994` = 0; `menu::load_slideshow_picture` with eax = 1;
//! `sound::play_car_stream`.
//! Its button in the menu table at 0x74B800.
//! Shadow mode runs the original alone.
//! Self-test: `menu::car_buttons::check_button`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::menu::car_buttons::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::previous_slideshow_car",
    rva(0x469B35),
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

// ---- Rust version ----

pub unsafe fn previous_slideshow_car(
    call: &mut dyn FnMut(Call) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        let car = game_read::<u32>(CAR_RVA).wrapping_sub(1);
        let car = if (car as i32) < 0 {
            game_read(LAST_CAR_RVA)
        } else {
            car
        };
        game_write(CAR_RVA, car);
        game_write(LEAVE_RVA, 0u32);
        game_write(PICTURE_RVA, 0u32);
        call(Call::Picture(1))?;
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
    if let Err(e) = unsafe { previous_slideshow_car(&mut |call| call_in_game(call, &entry)) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

// ---- self-test ----

const ORIGINAL_CODE: Original = Original {
    address: 0x469B35,
    len: 0x3B,
    entry: [0x60, 0xA1, 0x90, 0x59, 0x75, 0x00],
    calls: &[0x2F, 0x34],
};

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    check_button(&mut t, &ORIGINAL_CODE, &|call| unsafe {
        previous_slideshow_car(call)
    });
    t.finish()
}
