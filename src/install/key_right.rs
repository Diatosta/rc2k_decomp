//! `install::key_right` (0x4147C0): key 0x69 on an item (`[esi] & 0x2F`): kind 2 the
//! choice after (`install::choice_on`), kind 3 the value up (`install::value_up`).
//!
//! In:  esi the item
//! Out: bl the kind; as the helper leaves them (zero set), else the flags of `cmp bl, 3`
//! Caller `install::page_key`, reading zero. Shadow mode compares. Self-test: items of the
//! test's against the original.
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{CF, ZF, rd};
use crate::install::key_up::item_key;
use crate::race::viewer::{dispatch_regs, regs_entry};
use rc2k_core::install::items::ItemKey;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::key_right", rva(0x4147C0), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn key_right(entry: &Regs) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::install::items::ItemKey::helper`.
    item_key(entry, ItemKey::Right)
}

extern "C" fn dispatch(r: &mut Regs) {
    let target = (rd(r.esi + 8), 1usize);
    dispatch_regs(&HOOK, r, call_original, &[target], CF | ZF, &key_right);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &key_right)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::item_test::run(
        &mut t,
        call_original,
        &key_right,
        crate::install::item_test::any,
    );
    t.finish()
}
