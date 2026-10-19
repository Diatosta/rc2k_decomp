//! `install::value_up` (0x41489C): an item of a value (kind 3) raised by the key's repeats
//! (`g_listRepeats`), not above its most (`[esi+0Dh]`, signed compare).
//!
//! In:  esi the item (+8 the value's byte, +0xD the most)
//! Out: edx the value's address, ebx the most, ecx the new value; zero set, carry clear
//! Callers `install::key_right`. Shadow mode compares. Self-test: items of the test's
//! against the original.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{CF, REPEATS, ZF, g, rb, rd, wb};
use crate::race::viewer::dispatch_regs;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::value_up", rva(0x41489C), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn value_up(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.edx = rd(r.esi + 8);
    r.ebx = rb(r.esi + 0xD) as u32;
    r.ecx = (rb(r.edx) as u32).wrapping_add(g(REPEATS));
    if (r.ecx as i32) > r.ebx as i32 {
        r.ecx = r.ebx;
    }
    wb(r.edx, r.ecx as u8);
    r.set_zero(true);
    r.set_carry(false);
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    let target = (rd(r.esi + 8), 1usize);
    dispatch_regs(&HOOK, r, call_original, &[target], CF | ZF, &value_up);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::item_test::run(
        &mut t,
        call_original,
        &value_up,
        crate::install::item_test::with_bounds,
    );
    t.finish()
}
