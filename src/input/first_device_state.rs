//! `input::first_device_state` (0x409391): `input::device_state` of `g_inputDevices` (see
//! `input/axes.rs`).
//!
//! In:  nothing
//! Out: ecx, edx, eax, ebx the first device's four values; the rest kept
//!
//! Caller: `screens::read_controller`. Shadow mode: by state. Self-test: the game's device
//! (skipped without one).

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_read, hook_asm, rva};
use crate::input::axes::first_device_state;
use crate::input::device_state::left;
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("input::first_device_state", rva(0x409391), thunk, &ORIGINAL).self_test(self_test);

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
