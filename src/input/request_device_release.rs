//! `input::request_device_release` (0x409386): `g_inputCommand` 2: the input thread ends the
//! effects and gives both devices back (`input::device_command`). The same store as
//! `race::release_controls` (0x409425).
//!
//! In:  nothing
//! Out: every register kept
//! Caller: `menu::release_controller`. Self-test: the variable from 0, 1 and 3, put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::input::input_tick::COMMAND_RVA;
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "input::request_device_release",
    rva(0x409386),
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

/// The Rust version (see the header).
pub fn request_device_release() {
    unsafe { game_write(COMMAND_RVA, 2u32) };
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    request_device_release();
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[at(COMMAND_RVA)], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [at(COMMAND_RVA)];
    let saved = capture(&regions);
    for from in [0u32, 1, 3] {
        check_state(
            &mut t,
            &format!("from {from}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe { game_write(COMMAND_RVA, from) },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    t.finish()
}
