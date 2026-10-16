//! `race::create_controls` (0x40941A): `dword_4F7DE4` 3 (`race::load_stage`'s
//! "createcontrols"; what the 3 stands for is not read).
//!
//! In:  nothing
//! Out: every register preserved
//!
//! One caller, `race::load_stage`. Self-test: the variable from 0, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("race::create_controls", rva(0x40941A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const CONTROLS_RVA: usize = rva(0x4F7DE4); // dword_4F7DE4

pub fn create_controls() {
    unsafe { game_write(CONTROLS_RVA, 3u32) };
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    create_controls();
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[at(CONTROLS_RVA)], &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = [at(CONTROLS_RVA)];
    let saved = capture(&regions);
    check_state(
        &mut t,
        "from 0",
        call_original,
        Regs::SENTINEL,
        &|| unsafe { game_write(CONTROLS_RVA, 0u32) },
        &regions,
        &rust,
    );
    restore(&regions, &saved);
    t.finish()
}
