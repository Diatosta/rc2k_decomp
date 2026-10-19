//! `menu::keep_arcade_stage_best` (0x488213): the stage's best score kept in its record (see `menu/arcade_steps.rs`).
//!
//! In:  edi = the stage's record
//! Out: every register kept
//! Compared in shadow mode. Self-test: `arcade_steps::test_step`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::menu::arcade_steps::{dispatch_step, keep_arcade_stage_best, test_step};
use crate::screens::harness::Left;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::keep_arcade_stage_best",
    rva(0x488213),
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

fn rust(r: &Regs) -> Result<Left, Refused> {
    keep_arcade_stage_best(r.edi)?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_step(&HOOK, r, call_original, &rust);
}

fn self_test() -> (usize, usize) {
    test_step(&HOOK, call_original, &rust, &|stage| Regs {
        edi: stage,
        ..Regs::SENTINEL
    })
}
