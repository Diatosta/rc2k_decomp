//! `install::choice_back` (0x414826): an item of choices (kind 2) moved to the choice before
//! the chosen one that is not marked (bits 6, 7 of its second byte), round from the first
//! to the last; none: nothing changes.
//!
//! In:  esi the item (+1 the choices, +8 the chosen byte, +0xC the choices, 2 bytes each)
//! Out: edx the chosen byte's address, ebx the table, ecx the choice reached (byte); zero
//!      set, carry clear (`cmp eax, eax`)
//! The step down is `dec cl; jns`: from 0x80 .. 0xFF (and 0) it goes to the last choice.
//! A chosen byte past the choices with every choice marked never ends: refused (the
//! original goes round for ever). Callers `install::key_up`, `install::key_left`. Shadow
//! mode compares. Self-test: items of the test's against the original.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{CF, ZF, rb, rd, wb};
use crate::race::viewer::dispatch_regs;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::choice_back", rva(0x414826), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// Steps a choice may take before it is clear that the original goes round for ever.
pub const ROUND: usize = 0x202;

pub fn choice_back(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.edx = rd(r.esi + 8);
    r.ebx = rd(r.esi + 0xC);
    let mut cl = rb(r.edx);
    for _ in 0..ROUND {
        cl = cl.wrapping_sub(1);
        if cl & 0x80 != 0 {
            cl = rb(r.esi + 1).wrapping_sub(1);
        }
        let free = rb(r.ebx.wrapping_add(2 * cl as u32 + 1)) & 0xC0 == 0;
        if cl == rb(r.edx) || free {
            if cl != rb(r.edx) {
                wb(r.edx, cl);
            }
            r.ecx = cl as u32;
            r.set_zero(true);
            r.set_carry(false);
            return Ok(r);
        }
    }
    Err(BadInput("every choice marked and the chosen one past them").into())
}

extern "C" fn dispatch(r: &mut Regs) {
    let target = (rd(r.esi + 8), 1usize);
    dispatch_regs(&HOOK, r, call_original, &[target], CF | ZF, &choice_back);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::item_test::run(
        &mut t,
        call_original,
        &choice_back,
        crate::install::item_test::with_table,
    );
    t.finish()
}
