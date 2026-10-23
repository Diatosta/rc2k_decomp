//! `install::mode_page` (0x41422B): the installation-kind page. At the start (ebx 0) the
//! highlight (`g_listChoice`) is `g_mediaMode` (or 2 when it is 0) and `dword_51BA00` is 0.
//! At the end (ebx negative) `g_mediaMode` becomes the chosen answer (`byte_51D415`).
//!
//! In:  ebx the state, edi the list
//! Out: eax as the original leaves it; the flags of `test ebx, ebx` last
//! Caller `install::run_page`. Shadow mode compares.
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{CHOICE, MEDIA_MODE, PATH_PROBLEM, g, gb, p};
use crate::race::viewer::{dispatch_regs, regs_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::mode_page", rva(0x41422B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn mode_page(entry: &Regs) -> Regs {
    let mut r = *entry;
    if r.ebx == 0 {
        // Converted: `rc2k_core::install::pages::mode_highlight`.
        r.eax = rc2k_core::install::pages::mode_highlight(g(MEDIA_MODE));
        p(CHOICE, r.eax);
        p(PATH_PROBLEM, 0);
    }
    r.set_zero(r.ebx == 0);
    r.set_carry(false);
    if (r.ebx as i32) < 0 {
        r.eax = gb(crate::install::installer::LIST_ANSWER) as u32;
        p(MEDIA_MODE, r.eax);
    }
    r
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    Ok(mode_page(r))
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
