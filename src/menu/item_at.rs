//! `menu::item_at` (0x471CEF): the address of item number ebx of the current menu.
//!
//! In:  ebx = item number, from 1
//! Out: edi = the item, eax = ebx; everything else preserved
//! The current menu is `dword_75558D`; its items are 0x5C bytes each after a 4-byte head, so
//! item n is at menu + 4 + (n - 1) * 0x5C. The original counts up to ebx one item at a time
//! and never looks for the end of the menu. 14 callers.
//! Refuses item number 0, for which the original counts through all 32 bits.

use crate::errors::{BadInput, Refused};
use crate::framework::{EAX, EDI, RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{check_state, dispatch_state, state_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::item_at", rva(0x471CEF), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const CURRENT_MENU_RVA: usize = rva(0x75558D); // dword_75558D
pub use rc2k_core::menu::items::{ITEM_LEN, MENU_HEAD_LEN};

// ---- Rust version ----

pub fn item_at(number: u32) -> Result<u32, BadInput> {
    // Converted: `rc2k_core::menu::items::item_at`.
    Ok(rc2k_core::menu::items::item_at(number, || unsafe {
        game_read(CURRENT_MENU_RVA)
    })?)
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    let item = item_at(r.ebx)?;
    Ok(Left::nothing().with(EDI, item).with(EAX, r.ebx))
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &[], &rust);
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    state_entry(r, &rust)
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let saved: u32 = unsafe { game_read(CURRENT_MENU_RVA) };
    for menu in [0x0074_D24Cu32, 0x1000, 0xFFFF_FF00] {
        for number in [1u32, 2, 3, 10, 22, 100] {
            let before = Regs {
                ebx: number,
                ..Regs::SENTINEL
            };
            check_state(
                &mut t,
                &format!("menu={menu:#x} item={number}"),
                call_original,
                before,
                &|| unsafe { game_write(CURRENT_MENU_RVA, menu) },
                &[],
                &rust,
            );
        }
    }
    unsafe { game_write(CURRENT_MENU_RVA, saved) };
    t.finish()
}
