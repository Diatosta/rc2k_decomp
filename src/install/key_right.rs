//! `install::key_right` (0x4147C0): key 0x69 on an item (`[esi] & 0x2F`): kind 2 the
//! choice after (`install::choice_on`), kind 3 the value up (`install::value_up`).
//!
//! In:  esi the item
//! Out: bl the kind; as the helper leaves them (zero set), else the flags of `cmp bl, 3`
//! Caller `install::page_key`, reading zero. Shadow mode compares. Self-test: items of the
//! test's against the original.
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{CF, ZF, cmp, rb, rd};
use crate::race::viewer::dispatch_regs;
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
    let mut r = *entry;
    let kind = rb(r.esi) & 0x2F;
    r.ebx = r.ebx & !0xFF | kind as u32;
    if kind == 0x2 {
        return crate::install::choice_on::choice_on(&r);
    }
    if kind == 0x3 {
        return crate::install::value_up::value_up(&r);
    }
    cmp(&mut r, kind as u32, 0x3);
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    let target = (rd(r.esi + 8), 1usize);
    dispatch_regs(&HOOK, r, call_original, &[target], CF | ZF, &key_right);
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
