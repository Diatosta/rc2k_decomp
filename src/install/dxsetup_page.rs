//! `install::dxsetup_page` (0x41432F): at the page's end (ebx negative) with the answer
//! (`byte_51D415`) 1, `g_runDxSetup` is set so DirectX's set-up runs after the game.
//!
//! In:  ebx the state
//! Out: every register kept; the flags of its last test
//! Caller `install::run_page`. Shadow mode compares.
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::install::installer::{LIST_ANSWER, RUN_DXSETUP, gb, logic, p};
use crate::race::viewer::dispatch_regs;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("install::dxsetup_page", rva(0x41432F), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub fn dxsetup_page(entry: &Regs) -> Regs {
    let mut r = *entry;
    r.set_carry(false);
    r.set_zero((r.ebx as i32) >= 0);
    if (r.ebx as i32) < 0 {
        let answer = gb(LIST_ANSWER) as u32;
        logic(&mut r, answer.wrapping_sub(1));
        // Converted: `rc2k_core::install::pages::runs_dxsetup`.
        if rc2k_core::install::pages::runs_dxsetup(answer as u8) {
            p(RUN_DXSETUP, 1);
        }
    }
    r
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    Ok(dxsetup_page(r))
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = crate::install::page_action_test::globals();
    dispatch_regs(&HOOK, r, call_original, &regions, 0, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    crate::install::page_action_test::run(&mut t, call_original, &rust);
    t.finish()
}
