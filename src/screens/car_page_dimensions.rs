//! `screens::car_page_dimensions` (0x469CA0): the car specification's page 3 (see
//! `screens/car_pages.rs`).
//!
//! In:  edi = the car's record
//! Out: every register kept; the carry of the last text drawn
//! Self-test: `car_pages::test_page`.

use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::screens::car_pages::{dispatch_page, page_entry, test_page};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::car_page_dimensions",
    rva(0x469CA0),
    thunk,
    &ORIGINAL,
)
.self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_page(&HOOK, r, call_original, 3);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    page_entry("screens::car_page_dimensions", r, 3)
}

fn self_test() -> (usize, usize) {
    test_page(&HOOK, call_original, 3)
}
