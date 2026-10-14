//! `menu::disable_item` (0x46B4F6): an item of the current menu greyed out.
//!
//! In:  ebx = the item's number (from 1)
//! Out: every register preserved
//!
//! The item (`menu::item_at`): +0x48 = 0xFFFFFC1C (the moves up and down pass over it)
//! and its font +0x18 = the grey one at 0x51EA04. Its twin `menu::enable_item`
//! (0x46B50F) puts 0xFFFFFC1B and the font at 0x51E8E4 back. Eight callers each (the
//! player details screens: the controller's item when there is no controller).
//! Refuses item 0.
//! Self-test: items of a menu of the test's.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::menu::item_at::{CURRENT_MENU_RVA, ITEM_LEN, item_at};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::cell::RefCell;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::disable_item", rva(0x46B4F6), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

pub const CHOOSABLE: u32 = 0xFFFF_FC1B;
pub const NOT_CHOOSABLE: u32 = 0xFFFF_FC1C;
pub const GREY_FONT: usize = 0x51EA04;
pub const PLAIN_FONT: usize = 0x51E8E4;

// ---- Rust version ----

/// The item's +0x48 and font set: what both twins do.
pub fn set_item(number: u32, choosable: u32, font: usize) -> Result<(), Refused> {
    let item = item_at(number)?;
    unsafe {
        ((item + 0x48) as *mut u32).write_unaligned(choosable);
        ((item + 0x18) as *mut u32).write_unaligned(game_addr(crate::framework::rva(font)) as u32);
    }
    Ok(())
}

pub fn disable_item(number: u32) -> Result<(), Refused> {
    set_item(number, NOT_CHOOSABLE, GREY_FONT)
}

// ---- hook ----

/// The item the call writes, or nothing for item 0.
pub fn item_region(number: u32) -> Vec<Region> {
    item_at(number).map_or(Vec::new(), |item| vec![(item, ITEM_LEN as usize)])
}

fn rust(r: &Regs) -> Result<Left, Refused> {
    disable_item(r.ebx)?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let regions = item_region(r.ebx);
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

// ---- self-test ----

/// The twins' test: items 1 .. 12 of a menu of the test's.
pub fn check_item_setter(
    t: &mut SelfTest,
    call_original: unsafe extern "C" fn(*mut Regs),
    rust: &dyn Fn(&Regs) -> Result<Left, Refused>,
) {
    let saved = capture(&[at(CURRENT_MENU_RVA)]);
    let len = 4 + 12 * ITEM_LEN as usize;
    let menu = RefCell::new(vec![0xAAu8; len]);
    let at_menu = menu.borrow_mut().as_mut_ptr() as u32;
    for number in 1..=12u32 {
        check_state(
            t,
            &format!("item {number}"),
            call_original,
            Regs {
                ebx: number,
                ..Regs::SENTINEL
            },
            &|| unsafe {
                menu.borrow_mut().fill(0xAA);
                game_write(CURRENT_MENU_RVA, at_menu);
            },
            &[(at_menu, len)],
            rust,
        );
    }
    restore(&[at(CURRENT_MENU_RVA)], &saved);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    check_item_setter(&mut t, call_original, &rust);
    t.finish()
}
