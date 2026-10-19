//! `install::key_number` (0x4147DC): a number key (al 2 .. 0xA) on an item (`[esi] & 0xF`):
//! kind 2 that choice (`install::choice_by_number`), kind 3 that eighth of the value's
//! span (`install::value_by_number`); ecx the key less 2.
//!
//! In:  al the key, esi the item
//! Out: ecx the key less 2, bl the kind; as the helper leaves them (zero set), else the
//!      flags of `cmp bl, 3`
//! Caller `install::page_key`, reading zero. Shadow mode compares. Self-test: items of the
//! test's against the original, keys 2, 5, 0xA.
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{CF, ZF, cmp, rb, rd};
use crate::race::viewer::dispatch_regs;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::key_number", rva(0x4147DC), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn key_number(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.ecx = (r.eax as u8 as u32).wrapping_sub(2);
    let kind = rb(r.esi) & 0x0F;
    r.ebx = r.ebx & !0xFF | kind as u32;
    if kind == 0x2 {
        return crate::install::choice_by_number::choice_by_number(&r);
    }
    if kind == 0x3 {
        return crate::install::value_by_number::value_by_number(&r);
    }
    cmp(&mut r, kind as u32, 0x3);
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    let target = (rd(r.esi + 8), 1usize);
    dispatch_regs(&HOOK, r, call_original, &[target], CF | ZF, &key_number);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::item_test::run(
        &mut t,
        call_original,
        &key_number,
        crate::install::item_test::any,
    );
    t.finish()
}
