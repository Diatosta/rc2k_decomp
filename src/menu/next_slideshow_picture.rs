//! `menu::next_slideshow_picture` (0x469C1A): the slideshow's next picture.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! `dword_755994` + 1, 0 past 99 (signed); `menu::load_slideshow_picture` with
//! eax = 1.
//! Its button in the menu table at 0x74B85C.
//! Shadow mode runs the original alone.
//! Self-test: `menu::car_buttons::check_button`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva, shadow};
use crate::menu::car_buttons::*;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::next_slideshow_picture",
    rva(0x469C1A),
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

pub unsafe fn next_slideshow_picture(
    call: &mut dyn FnMut(Call) -> Result<(), Refused>,
) -> Result<(), Refused> {
    unsafe {
        let picture = game_read::<u32>(PICTURE_RVA).wrapping_add(1);
        game_write(PICTURE_RVA, if (picture as i32) > 99 { 0 } else { picture });
        call(Call::Picture(1))
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
    if let Err(e) = unsafe { next_slideshow_picture(&mut |call| call_in_game(call, &entry)) } {
        unsafe { crate::fatal::raise::refused(&HOOK, e, r, call_original) }
    }
}

// ---- self-test ----

const ORIGINAL_CODE: Original = Original {
    address: 0x469C1A,
    len: 0x26,
    entry: [0x60, 0xFF, 0x05, 0x94, 0x59, 0x75],
    calls: &[0x1F],
};

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    check_button(&mut t, &ORIGINAL_CODE, &|call| unsafe {
        next_slideshow_picture(call)
    });
    t.finish()
}
