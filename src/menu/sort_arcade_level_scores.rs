//! `menu::sort_arcade_level_scores` (0x48A2AE): the level's best scores sorted up from the tenth (see `menu/arcade_steps.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! Compared in shadow mode. Self-test: `arcade_steps::test_step`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::menu::arcade_steps::{dispatch_step, sort_arcade_level_scores, test_step};
use crate::screens::harness::Left;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::sort_arcade_level_scores",
    rva(0x48A2AE),
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
    sort_arcade_level_scores()?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_step(&HOOK, r, call_original, &rust);
}

fn self_test() -> (usize, usize) {
    test_step(&HOOK, call_original, &rust, &|_| Regs::SENTINEL)
}
