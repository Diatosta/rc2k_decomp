//! `race::set_player_car_kinds` (0x43670F): the two players' blocks' +0x24, +0x2C, +0x38
//! from (`g_raceCarKinds`, `dword_6235FC`, `dword_623878`) and (`dword_62385C`,
//! `dword_623600`, `dword_62387C`).
//!
//! In:  nothing
//! Out: every register preserved (`push` / `pop` pairs)
//!
//! See `race/race_cars.rs`. Callers: `sub_43603C` and its four set-ups. Shadow mode: the
//! Rust version, the blocks put back, the original's, compared. Self-test: the game's
//! blocks filled with a pattern and kinds of the test's, both versions, put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::race_cars::*;
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "race::set_player_car_kinds",
    rva(0x43670F),
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

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { set_player_car_kinds() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &kind_regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let kept = saved_regions();
    let saved = capture(&kept);
    let regions = kind_regions();
    for seed in [0u32, 0xFFFF_FFFF, 0x0BAD_F00D] {
        check_state(
            &mut t,
            &format!("seed {seed:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| prepare(seed, 0, 2, 2, [0, 0]),
            &regions,
            &rust,
        );
    }
    restore(&kept, &saved);
    t.finish()
}
