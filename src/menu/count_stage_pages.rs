//! `menu::count_stage_pages` (0x480032): how many pages the stage has (see
//! `menu/stage_view_steps.rs`).
//!
//! In:  nothing
//! Out: every register kept
//! Called by `menu::view_stage_loop`. Compared in shadow mode. Self-test: records of the
//! test's at the start of `g_recordTable`, put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::menu::stage_view_steps::{count_stage_pages, dispatch_plain, plain_entry, test_count};
use crate::screens::harness::Left;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::count_stage_pages", rva(0x480032), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    count_stage_pages()?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_plain(&HOOK, r, call_original, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    plain_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    test_count(&HOOK, call_original)
}
