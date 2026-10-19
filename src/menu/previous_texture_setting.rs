//! `menu::previous_texture_setting` (0x47012B): `dword_755664` one less, 0 to 2 (see `menu/option_steps.rs`).
//!
//! In:  nothing
//! Out: eax the value
//! A left/right handler of an options page's item. Compared in shadow mode. Self-test:
//! `option_steps::test_plain`.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::menu::option_steps::{dispatch_plain, step_texture_setting, test_plain};
use crate::screens::harness::Left;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "menu::previous_texture_setting",
    rva(0x47012B),
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
    Ok(step_texture_setting(false))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_plain(&HOOK, r, call_original, &rust);
}

fn self_test() -> (usize, usize) {
    test_plain(&HOOK, call_original, &rust)
}
