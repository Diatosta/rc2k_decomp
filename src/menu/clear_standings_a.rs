//! `menu::clear_standings_a` (0x4A5A14): each entry of standings table A (see `menu::standings`)
//! given +0x44 0.
//!
//! In:  nothing
//! Out: every register kept
//!
//! Callers: the event drivers (`sub_4A0E60`, ..). Refuses a table without its end entry,
//! before anything. Self-test: `standings::test_clear` on a table of the test's.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, dispatch_state};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::clear_standings_a", rva(0x4A5A14), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn clear_standings_a(entry: &Regs) -> Result<Regs, Refused> {
    let first: u32 = unsafe { crate::framework::game_read(TABLE_A_RVA) };
    let all = entries(first, A_ENTRY)?;
    // Converted: `rc2k_core::menu::standings::A_FIGURE`, stored in each entry.
    use rc2k_core::menu::standings::A_FIGURE;
    unsafe { table(first, A_ENTRY, all.len() as u32) }.store_all(&[(A_FIGURE, 0)]);
    Ok(*entry)
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { clear_standings_a(r) }?))
}

fn regions(r: &Regs) -> Vec<Region> {
    let _ = r;
    let first: u32 = unsafe { crate::framework::game_read(TABLE_A_RVA) };
    match entries(first, A_ENTRY) {
        Ok(all) => vec![(first, (all.len() as u32 * A_ENTRY) as usize)],
        Err(_) => vec![],
    }
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = regions(r);
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    test_clear(&HOOK, call_original, TABLE_A_RVA, A_ENTRY, &rust)
}
