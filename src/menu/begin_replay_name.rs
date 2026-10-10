//! `menu::begin_replay_name` (0x46F715): starts typing the name of the highlighted replay:
//! the name is copied to the text entry's backup and cleared, the entry's length is 0 and
//! `dword_7556A8` (an entry is going on) is 1.
//! It first logs "getrepsaveslotname result=<name>".
//!
//! In:  nothing
//! Out: every register preserved
//! It writes "Backed up name=<the name>" to the game's log (`logfile::log_write`). In six menu tables. Refuses a name past the list.
//! Shadow mode runs the original first and then the Rust version without the log lines, so
//! that nothing is logged twice. The self-test runs without pausing the game: the original
//! logs (as does the Rust version there, so each test line appears twice in the game's log),
//! and a paused game thread could be holding the log's lock.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::menu::name_lists::{NAME_LEN, REPLAY_PAGE_RVA, replay_name};
use crate::menu::text_entry::{
    ENTERING_RVA, LENGTH_RVA, backup, backup_region, begin_logged, log_l1, set_length,
};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_replayed_state, restore};
use crate::service::state::MENU_ITEM_RVA;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new("menu::begin_replay_name", rva(0x46F715), thunk, &ORIGINAL)
    .self_test(self_test)
    .self_test_unpaused();

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

/// `log` = false leaves the log lines out.
pub unsafe fn begin_replay_name(log: bool) -> Result<(), BadInput> {
    let name = replay_name()?;
    unsafe {
        if log {
            log_l1(game_addr(rva(0x4DE1F3)) as *const u8, false); // aGetrepsaveslot
            log_l1(name, true);
        }
        begin_logged(name, log);
    }
    Ok(())
}

// ---- hook ----

fn regions() -> Vec<Region> {
    let mut regions = vec![backup_region(), at(LENGTH_RVA), at(ENTERING_RVA)];
    if let Ok(name) = replay_name() {
        regions.push((name as u32, NAME_LEN));
    }
    regions
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { begin_replay_name(true) }?;
    Ok(Left::nothing())
}

fn quiet(_: &Regs) -> Result<Left, Refused> {
    unsafe { begin_replay_name(false) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_replayed_state(&HOOK, r, call_original, &regions(), &rust, &quiet);
}

// ---- self-test: the last name of the list, which is put back ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let inputs = [at(REPLAY_PAGE_RVA), at(MENU_ITEM_RVA)];
    let saved_inputs = capture(&inputs);
    unsafe {
        game_write(REPLAY_PAGE_RVA, 4u32);
        game_write(MENU_ITEM_RVA, 20u32);
    }
    let regions = regions();
    let saved = capture(&regions);
    if let Ok(name) = replay_name() {
        for (n, text) in [&b"selftest name\0"[..], &b"\0"[..]]
            .into_iter()
            .enumerate()
        {
            check_state(
                &mut t,
                &format!("name {n}"),
                call_original,
                Regs::SENTINEL,
                &|| unsafe {
                    name.write_bytes(0, NAME_LEN);
                    std::ptr::copy_nonoverlapping(text.as_ptr(), name, text.len());
                    backup().write_bytes(0, 0x1F);
                    set_length(7);
                    game_write(ENTERING_RVA, 0x5EEDu32);
                },
                &regions,
                &rust,
            );
        }
    }
    restore(&regions, &saved);
    restore(&inputs, &saved_inputs);
    t.finish()
}
