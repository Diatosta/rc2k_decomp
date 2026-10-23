//! `install::ignore_room` (0x4142DA): the not-enough-room page's "go on anyway" action:
//! `dword_51BA04` 1, the answer (`byte_51D415`) 1 and the page left.
//!
//! In:  nothing
//! Out: every register kept; zero set, carry clear (`cmp eax, eax`)
//! An item action (kind 6). Caller `install::item_action` through a page's item. Shadow
//! mode compares.
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{IGNORE_ROOM, LIST_ANSWER, LIST_LEAVE, p, pb};
use crate::race::viewer::{dispatch_regs, regs_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::ignore_room", rva(0x4142DA), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn ignore_room(entry: &Regs) -> Regs {
    let mut r = *entry;
    p(IGNORE_ROOM, 1);
    pb(LIST_ANSWER, 1);
    pb(LIST_LEAVE, 1);
    r.set_zero(true);
    r.set_carry(false);
    r
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    Ok(ignore_room(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = crate::install::page_action_test::globals();
    dispatch_regs(&HOOK, r, call_original, &regions, 0, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::page_action_test::run(&mut t, call_original, &rust);
    t.finish()
}
