//! `install::key_down` (0x414794): key 0x6B on an item: for choices shown upright (kind
//! 0x22 of `[esi] & 0x2F`) the choice after (`install::choice_on`).
//!
//! In:  esi the item
//! Out: bl the kind (`[esi] & 0x2F`); with choices as `choice_on` leaves them (zero set),
//!      else the flags of `cmp bl, 22h` (zero clear)
//! Caller `install::page_key`, reading zero. Shadow mode compares. Self-test: items of the
//! test's against the original.
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{CF, ZF, rd};
use crate::install::key_up::item_key;
use crate::race::viewer::dispatch_regs;
use rc2k_core::install::items::ItemKey;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::key_down", rva(0x414794), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn key_down(entry: &Regs) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::install::items::ItemKey::helper`.
    item_key(entry, ItemKey::Down)
}

extern "C" fn dispatch(r: &mut Regs) {
    let target = (rd(r.esi + 8), 1usize);
    dispatch_regs(&HOOK, r, call_original, &[target], CF | ZF, &key_down);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::item_test::run(
        &mut t,
        call_original,
        &key_down,
        crate::install::item_test::any,
    );
    t.finish()
}
