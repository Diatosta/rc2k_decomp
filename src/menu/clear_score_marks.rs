//! `menu::clear_score_marks` (0x4A919A): the 50 best-score entries (`dword_7556D4`, 0x3C
//! each; see `saves::default_best_scores`) given +0x14 0 and +4 0.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`)
//!
//! Callers: the event drivers. Refuses a null table, before anything. Self-test: the
//! original and the Rust version on a table of the test's; put back.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::menu::standings::*;
use crate::race::stage_prefs::left_of;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::clear_score_marks", rva(0x4A919A), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const TABLE_RVA: usize = rva(0x7556D4); // dword_7556D4
const ENTRIES: u32 = 50;

/// The Rust version (see the header): the registers it leaves.
pub unsafe fn clear_score_marks(entry: &Regs) -> Result<Regs, Refused> {
    let first: u32 = unsafe { game_read(TABLE_RVA) };
    if first == 0 {
        return Err(BadInput("no best-score table").into());
    }
    // Converted: `rc2k_core::menu::standings::BEST_MARKS`, stored in each entry.
    unsafe { table(first, 0x3C, ENTRIES) }.store_all(&rc2k_core::menu::standings::BEST_MARKS);
    Ok(*entry)
}

fn regions() -> Vec<Region> {
    let first: u32 = unsafe { game_read(TABLE_RVA) };
    if first == 0 {
        vec![]
    } else {
        vec![(first, (ENTRIES * 0x3C) as usize)]
    }
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    Ok(left_of(&unsafe { clear_score_marks(r) }?))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let mut table = vec![0xA5A5_A5A5u32; (ENTRIES * 0x3C / 4) as usize];
    let tp = table.as_mut_ptr() as u32;
    let vars = vec![at(TABLE_RVA)];
    let saved = capture(&vars);
    let regions = vec![(tp, (ENTRIES * 0x3C) as usize)];
    check_state(
        &mut t,
        "a test table",
        call_original,
        Regs::SENTINEL,
        &|| unsafe { game_write(TABLE_RVA, tp) },
        &regions,
        &rust,
    );
    restore(&vars, &saved);
    drop(table);
    t.finish()
}
