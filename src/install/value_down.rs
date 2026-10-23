//! `install::value_down` (0x414881): an item of a value (kind 3) lowered by the key's
//! repeats (`g_listRepeats`), not below its least (`[esi+0Ch]`, signed compare).
//!
//! In:  esi the item (+8 the value's byte, +0xC the least)
//! Out: edx the value's address, ebx the least, ecx the new value; zero set, carry clear
//! Callers `install::key_left`. Shadow mode compares. Self-test: items of the test's
//! against the original.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{CF, REPEATS, ZF, g, rb, rd, wb};
use crate::race::viewer::{dispatch_regs, regs_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::value_down", rva(0x414881), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn value_down(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.edx = rd(r.esi + 8);
    r.ebx = rb(r.esi + 0xC) as u32;
    // Converted: `rc2k_core::install::items::value_down`.
    r.ecx = rc2k_core::install::items::value_down(rb(r.edx), g(REPEATS), r.ebx as u8);
    wb(r.edx, r.ecx as u8);
    r.set_zero(true);
    r.set_carry(false);
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    let target = (rd(r.esi + 8), 1usize);
    dispatch_regs(&HOOK, r, call_original, &[target], CF | ZF, &value_down);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &value_down)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::item_test::run(
        &mut t,
        call_original,
        &value_down,
        crate::install::item_test::with_bounds,
    );
    t.finish()
}
