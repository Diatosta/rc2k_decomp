//! `service::begin_setup_name` (0x46EE81): starts typing the name of the highlighted setup
//! slot: the name is copied to the text entry's backup and cleared, the entry's length is 0
//! and `dword_7556A8` (an entry is going on) is 1.
//!
//! In:  nothing
//! Out: every register preserved
//! The action of the slots on the "save setup" menu (six menu tables point at it); from then
//! on the screen loop runs `edit_setup_name`. Refuses a slot past 99.

use crate::errors::BadInput;
use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::menu::backup_text::backup_text;
use crate::menu::clear_text::clear_text;
use crate::menu::text_entry::{ENTERING_RVA, LENGTH_RVA, backup_region, set_length};
use crate::screens::harness::Left;
use crate::service::backspace_setup_name::{TEST_ITEM, TEST_PAGE};
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use crate::service::load_setup_slot::{SLOT_PAGE_RVA, highlighted_slot};
use crate::service::read_setup_slot::{NAME_LEN, SLOT_COUNT, slot_name};
use crate::service::state::MENU_ITEM_RVA;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("service::begin_setup_name", rva(0x46EE81), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- Rust version ----

pub unsafe fn begin_setup_name() -> Result<(), BadInput> {
    let slot = highlighted_slot();
    if slot >= SLOT_COUNT {
        return Err(BadInput("slot past the name table"));
    }
    unsafe {
        backup_text(slot_name(slot));
        clear_text(slot_name(slot));
        game_write(ENTERING_RVA, 1u32);
    }
    set_length(0);
    Ok(())
}

// ---- hook ----

fn regions() -> Vec<Region> {
    let mut regions = vec![backup_region(), at(LENGTH_RVA), at(ENTERING_RVA)];
    let slot = highlighted_slot();
    if slot < SLOT_COUNT {
        regions.push((slot_name(slot) as u32, NAME_LEN));
    }
    regions
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { begin_setup_name() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test: slot 99 (page 3, item 10), whose name is put back ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let inputs = [at(SLOT_PAGE_RVA), at(MENU_ITEM_RVA)];
    let saved_inputs = capture(&inputs);
    unsafe {
        game_write(SLOT_PAGE_RVA, TEST_PAGE);
        game_write(MENU_ITEM_RVA, TEST_ITEM);
    }
    let regions = regions();
    let saved = capture(&regions);
    for (n, name) in [&b"a name\0"[..], &b"\0"[..], &[b'f'; NAME_LEN][..]]
        .into_iter()
        .enumerate()
    {
        check_state(
            &mut t,
            &format!("name {n}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                slot_name(99).write_bytes(0xAA, NAME_LEN);
                std::ptr::copy_nonoverlapping(name.as_ptr(), slot_name(99), name.len());
                crate::menu::text_entry::backup().write_bytes(0xBB, 0x1F);
                set_length(7);
                game_write(ENTERING_RVA, 0x5EEDu32);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    restore(&inputs, &saved_inputs);
    t.finish()
}
