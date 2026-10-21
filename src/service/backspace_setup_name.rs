//! `service::backspace_setup_name` (0x46D63C): takes the last character off the name of the
//! highlighted setup slot: if the text entry's length is not 0, it is lowered by one and a
//! NUL is written there.
//!
//! In:  nothing
//! Out: every register preserved
//! Refuses a slot past the table and a length outside 0 .. 0x1F (the original writes
//! wherever the length points).

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_write, hook_asm, rva};
use crate::menu::text_entry::{LENGTH_RVA, length, set_length};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use crate::service::load_setup_slot::{SLOT_PAGE_RVA, highlighted_slot};
use crate::service::read_setup_slot::{NAME_LEN, SLOT_COUNT, slot_name};
use crate::service::state::MENU_ITEM_RVA;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "service::backspace_setup_name",
    rva(0x46D63C),
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

// ---- Rust version ----

/// The name of the highlighted slot, and the text entry's length if it is usable.
pub fn entry() -> Result<(*mut u8, usize), BadInput> {
    let slot = highlighted_slot();
    if slot >= SLOT_COUNT {
        return Err(BadInput("slot past the name table"));
    }
    let len = length();
    if !(0..NAME_LEN as i32).contains(&len) {
        return Err(BadInput("text length outside the name"));
    }
    Ok((slot_name(slot), len as usize))
}

pub unsafe fn backspace_setup_name() -> Result<(), BadInput> {
    // Converted: `rc2k_core::service::setups::backspace`.
    let (name, len) = entry()?;
    if len != 0 {
        let name = unsafe { std::slice::from_raw_parts_mut(name, NAME_LEN) };
        set_length(rc2k_core::service::setups::backspace(name, len) as i32);
    }
    Ok(())
}

// ---- hook ----

fn regions() -> Vec<Region> {
    let mut regions = vec![at(LENGTH_RVA)];
    let slot = highlighted_slot();
    if slot < SLOT_COUNT {
        regions.push((slot_name(slot) as u32, NAME_LEN));
    }
    regions
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { backspace_setup_name() }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test: slot 99 (page 3, item 10), whose name is put back ----

pub const TEST_PAGE: u32 = 3;
pub const TEST_ITEM: u32 = 10;

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
    for len in [0i32, 1, 5, 30, 31] {
        check_state(
            &mut t,
            &format!("length={len}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                slot_name(99).write_bytes(b'n', NAME_LEN);
                set_length(len);
            },
            &regions,
            &rust,
        );
    }
    restore(&regions, &saved);
    restore(&inputs, &saved_inputs);
    t.finish()
}
