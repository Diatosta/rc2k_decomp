//! `install::mark_main_page` (0x4141EF): the main page's items marked by the installation's
//! state. Not this version (`dword_51B9C8` != 1): item +0x28 gets 0x80 (unusable); no
//! installation (`g_installStatus` 0): item +0x38 gets 0x80; registration not offered
//! (`dword_51B9E4` != 1): item +0x48 gets 0x40 (hidden). If the title word (`[edi+2Ah]`) is
//! 0xBD (the update script): +0x28 loses 0x80, +0x38 and +0x48 get 0x40.
//!
//! In:  edi the list
//! Out: every register kept; the flags of its last test (no caller reads them)
//! A page action; `ebx` is ignored. Caller `install::run_page`. Shadow mode compares.
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{OFFER_REGISTRATION, SAME_VERSION, STATUS, cmp, g, gb, rb, rw, wb};
use crate::race::viewer::{dispatch_regs, regs_entry};
use rc2k_core::install::pages;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::mark_main_page", rva(0x4141EF), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn mark_main_page(entry: &Regs) -> Regs {
    let mut r = *entry;
    let title = rw(r.edi + 0x2A);
    // Converted: `rc2k_core::install::pages::main_page_marks`.
    let marks = pages::main_page_marks(
        g(SAME_VERSION) == 1,
        gb(STATUS) != 0,
        g(OFFER_REGISTRATION) == 1,
        title,
    );
    for (offset, or, and) in marks {
        wb(r.edi + offset, (rb(r.edi + offset) | or) & and);
    }
    cmp(&mut r, title as u32, pages::UPDATE_TITLE as u32);
    r
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    Ok(mark_main_page(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = crate::install::page_action_test::globals();
    dispatch_regs(&HOOK, r, call_original, &regions, 0, &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &rust)
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::page_action_test::run(&mut t, call_original, &rust);
    t.finish()
}
