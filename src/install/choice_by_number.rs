//! `install::choice_by_number` (0x41486A): an item of choices (kind 2) set to choice ecx (a
//! number key's), if there is one and it is not marked.
//!
//! In:  esi the item, ecx the choice (`install::key_number`: the key less 2)
//! Out: edx the chosen byte's address, ebx the table; zero set, carry clear
//! The compare is of cl with the choices; the mark is read at `[ebx+ecx*2+1]`, all of ecx.
//! Callers `install::key_number`. Shadow mode compares. Self-test: items of the test's
//! against the original, ecx 0 .. 8.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{CF, ZF, rb, rd, wb};
use crate::race::viewer::{dispatch_regs, regs_entry};
use rc2k_core::install::items::{self, MARKS};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::choice_by_number", rva(0x41486A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn choice_by_number(entry: &Regs) -> Result<Regs, Refused> {
    let mut r = *entry;
    r.edx = rd(r.esi + 8);
    r.ebx = rd(r.esi + 0xC);
    // Converted: `rc2k_core::install::items::choice_by_number`.
    let free = || rb(r.ebx.wrapping_add(r.ecx.wrapping_mul(2)).wrapping_add(1)) & MARKS == 0;
    if let Some(choice) = items::choice_by_number(r.ecx, rb(r.esi + 1), free) {
        wb(r.edx, choice);
    }
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
        &choice_by_number,
    );
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &choice_by_number)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::item_test::run(
        &mut t,
        call_original,
        &choice_by_number,
        crate::install::item_test::with_table,
    );
    t.finish()
}
