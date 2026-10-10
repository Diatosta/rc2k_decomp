//! `menu::begin_savegame_name` (0x46D571): starts typing the name of the highlighted saved game:
//! the name is copied to the text entry's backup and cleared, the entry's length is 0 and
//! `dword_7556A8` (an entry is going on) is 1.
//!
//! In:  nothing
//! Out: every register preserved
//! It writes "Backed up name=<the name>" to the game's log, through the original's unported
//! log entry points (`call_game`). In six menu tables. Refuses a name past the list.
//! Shadow mode runs the original first and then the Rust version without the log lines, so
//! that nothing is logged twice. The self-test runs without pausing the game: the original
//! logs (as does the Rust version there, so each test line appears twice in the game's log),
//! and a paused game thread could be holding the log's lock.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::menu::name_lists::{NAME_LEN, SAVEGAME_PAGE_RVA, savegame_name};
use crate::menu::text_entry::{
    ENTERING_RVA, LENGTH_RVA, backup, backup_region, begin_logged, set_length,
};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_replayed_state, restore};
use crate::service::state::MENU_ITEM_RVA;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::begin_savegame_name", rva(0x46D571), thunk, &ORIGINAL)
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
pub unsafe fn begin_savegame_name(log: bool) -> Result<(), BadInput> {
    let name = savegame_name()?;
    unsafe {
        begin_logged(name, log);
    }
    Ok(())
}

// ---- hook ----

fn regions() -> Vec<Region> {
    let mut regions = vec![backup_region(), at(LENGTH_RVA), at(ENTERING_RVA)];
    if let Ok(name) = savegame_name() {
        regions.push((name as u32, NAME_LEN));
    }
    regions
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { begin_savegame_name(true) }?;
    Ok(Left::nothing())
}

fn quiet(_: &Regs) -> Result<Left, Refused> {
    unsafe { begin_savegame_name(false) }?;
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
    let inputs = [at(SAVEGAME_PAGE_RVA), at(MENU_ITEM_RVA)];
    let saved_inputs = capture(&inputs);
    unsafe {
        game_write(SAVEGAME_PAGE_RVA, 9u32);
        game_write(MENU_ITEM_RVA, 10u32);
    }
    let regions = regions();
    let saved = capture(&regions);
    if let Ok(name) = savegame_name() {
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
