//! `install::room_page` (0x4142C7): the not-enough-room page. At the start (ebx 0)
//! `dword_51BA04` (ignore the room) is set to 0.
//!
//! In:  ebx the state
//! Out: every register kept; the flags of `test ebx, ebx` last
//! Caller `install::run_page`. Shadow mode compares.
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{IGNORE_ROOM, p};
use crate::race::viewer::dispatch_regs;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::room_page", rva(0x4142C7), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn room_page(entry: &Regs) -> Regs {
    let mut r = *entry;
    if r.ebx == 0 {
        p(IGNORE_ROOM, 0);
    }
    r.set_zero(r.ebx == 0);
    r.set_carry(false);
    r
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    Ok(room_page(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = crate::install::page_action_test::globals();
    dispatch_regs(&HOOK, r, call_original, &regions, 0, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::page_action_test::run(&mut t, call_original, &rust);
    t.finish()
}
