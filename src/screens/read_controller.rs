//! `screens::read_controller` (0x46B4AE): `call sub_409391; retn`: the first controller's
//! four values (`input::first_device_state`).
//!
//! In:  nothing
//! Out: ecx, edx, eax, ebx the values; the rest kept
//!
//! It is its call: private copy (6, the call relocated), manual patch. Callers: 0x46A78A,
//! `screens::learn_axis`. Shadow mode: by state. Self-test: the game's device (skipped
//! without one).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, hook_asm, rva};
use crate::input::axes::first_device_state;
use crate::input::device_state::left;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("screens::read_controller", rva(0x46B4AE), thunk, &ORIGINAL)
        .private_copy(0x6)
        .relocate_calls(&[0x0])
        .manual_patch()
        .self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    if unsafe { game_read::<u32>(rva(0x4F6EE8)) } == 0 {
        return Err(crate::errors::BadInput("no input device").into());
    }
    Ok(left(unsafe { first_device_state() }))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if unsafe { game_read::<u32>(rva(0x4F6EE8)) } == 0 {
        return t.skip("no input device");
    }
    check_state(
        &mut t,
        "the first device",
        call_original,
        Regs::SENTINEL,
        &|| {},
        &[],
        &rust,
    );
    t.finish()
}
