//! `menu::list_item_unusable` (0x414C1B): whether the highlighted item of a list menu
//! (`dword_51D407`, 16 bytes from +0x18) cannot be chosen: its first byte's bits 6 and 7
//! exactly 0x40 or 0x80, or its kind (low nibble) 5.
//!
//! In:  edi the list
//! Out: carry set when it cannot be chosen; every register kept (eax pushed)
//!
//! Callers: `race::replay_viewer` (three), `sub_4144CF` (three), reading carry. Shadow mode
//! compares. Self-test: every mark and kind of the item against the original.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::race::viewer::{CF, CHOICE, check_original, dispatch_regs, get, put, regs_entry};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::list_item_unusable", rva(0x414C1B), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version: carry.
pub unsafe fn list_item_unusable(list: u32) -> bool {
    let at = get(CHOICE)
        .wrapping_mul(0x10)
        .wrapping_add(list)
        .wrapping_add(0x18);
    // Converted: `rc2k_core::menu::list_menu::unusable`.
    rc2k_core::menu::list_menu::unusable(unsafe { (at as *const u8).read() })
}

fn rust(r: &Regs) -> Result<Regs, Refused> {
    let mut left = *r;
    left.set_carry(unsafe { list_item_unusable(r.edi) });
    Ok(left)
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_regs(&HOOK, r, call_original, &[], CF, &rust);
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
    let saved = crate::race::viewer::Saved::new(&[crate::race::viewer::dword(CHOICE)]);
    let mut list = [0u8; 0x18 + 0x30];
    let list_at = list.as_mut_ptr() as u32;
    for byte in [0x00u8, 0x40, 0x80, 0xC0, 0x05, 0x45, 0xC5, 0x02, 0x15, 0x3F] {
        for choice in [0u32, 2] {
            list[0x18 + 0x10 * choice as usize] = byte;
            check_original(
                &mut t,
                &format!("item {choice} byte {byte:#x}"),
                call_original,
                Regs {
                    edi: list_at,
                    ..Regs::SENTINEL
                },
                &[],
                CF,
                &|| put(CHOICE, choice),
                &rust,
            );
        }
    }
    drop(saved);
    t.finish()
}
