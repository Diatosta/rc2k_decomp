//! `menu::next_texture_setting` (0x47015B): `dword_755664` one more, 2 to 0 (see `menu/option_steps.rs`).
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
    "menu::next_texture_setting",
    rva(0x47015B),
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
    Ok(step_texture_setting(true))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_plain(&HOOK, r, call_original, &rust);
}

fn self_test() -> (usize, usize) {
    test_plain(&HOOK, call_original, &rust)
}
