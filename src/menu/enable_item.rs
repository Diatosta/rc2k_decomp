//! `menu::enable_item` (0x46B50F): an item of the current menu made choosable again.
//!
//! In:  ebx = the item's number (from 1)
//! Out: every register preserved
//!
//! The item: +0x48 = 0xFFFFFC1B and its font +0x18 = the one at 0x51E8E4. The twin of
//! `menu::disable_item`, which has the details. Eight callers.
//! Refuses item 0.
//! Self-test: items of a menu of the test's.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::menu::disable_item::{check_item_setter, item_region, set_item};
use crate::screens::harness::Left;
use crate::service::harness::{dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::enable_item", rva(0x46B50F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub fn enable_item(number: u32) -> Result<(), Refused> {
    // Converted: `rc2k_core::menu::items::choosable`.
    set_item(number, true)
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    enable_item(r.ebx)?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = item_region(r.ebx);
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    check_item_setter(&mut t, call_original, &rust);
    t.finish()
}
