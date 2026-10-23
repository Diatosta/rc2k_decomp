//! `install::value_by_number` (0x4148B7): an item of a value (kind 3) set by a number key to
//! its least plus (most - least) x ecx / 8 (`imul`, `idiv` by 8, towards 0).
//!
//! In:  esi the item (+8 the value's byte, +0xC the least, +0xD the most), ecx the number
//! Out: eax kept; ebx the least, ecx 8, edx the value's address; zero set, carry clear
//! Callers `install::key_number` (ecx the key less 2: 0 .. 8). Shadow mode compares.
//! Self-test: items of the test's against the original.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{CF, ZF, rb, rd, wb};
use crate::race::viewer::{dispatch_regs, regs_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::value_by_number", rva(0x4148B7), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn value_by_number(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    let least = rb(r.esi + 0xC);
    // Converted: `rc2k_core::install::items::value_by_number`.
    let value = rc2k_core::install::items::value_by_number(least, rb(r.esi + 0xD), r.ecx)?;
    r.ebx = least as u32;
    r.ecx = 8;
    r.edx = rd(r.esi + 8);
    wb(r.edx, value);
    r.set_zero(true);
    r.set_carry(false);
    Ok(r)
}

extern "C" fn dispatch(r: &mut Regs) {
    let target = (rd(r.esi + 8), 1usize);
    dispatch_regs(
        &HOOK,
        r,
        call_original,
        &[target],
        CF | ZF,
        &value_by_number,
    );
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &value_by_number)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::item_test::run(
        &mut t,
        call_original,
        &value_by_number,
        crate::install::item_test::with_bounds,
    );
    t.finish()
}
