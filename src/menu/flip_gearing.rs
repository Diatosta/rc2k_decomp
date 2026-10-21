//! `menu::flip_gearing` (0x469916): `dword_7554F0` flipped (`xor 1`) and its text into
//! `byte_A8BF54`: string 0x21F, or 0x221 when it is then 0, and a NUL.
//!
//! In:  nothing
//! Out: every register kept (`pusha` .. `popa`; the original loads eax just before the
//!      `popa`)
//!
//! Unlike `menu::next_gearing` it does not touch the player's record. Reached from a menu
//! table (0x74BA9C, a left/right handler of an item of the car choice). Compared in shadow
//! mode. Self-test: the game's strings, `dword_7554F0` from -1 to 3, put back.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::screens::gearing_text::{KEPT_RVA, TEXT_LEN, TEXT_RVA, regions};
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use crate::strings::append_string_by_id::push_string;
use crate::strings::string_table::blob;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::flip_gearing", rva(0x469916), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

/// The Rust version (see the header).
pub unsafe fn flip_gearing() {
    unsafe {
        // Converted: `rc2k_core::menu::steps` (`flip`, `gearing_switch_text`).
        use rc2k_core::menu::steps::{flip, gearing_switch_text};
        let kept = flip(game_read::<u32>(KEPT_RVA));
        game_write(KEPT_RVA, kept);
        let mut text = Vec::new();
        push_string(&mut text, gearing_switch_text(kept));
        text.push(0);
        std::ptr::copy_nonoverlapping(text.as_ptr(), game_addr(TEXT_RVA) as *mut u8, text.len());
    }
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { flip_gearing() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if blob().is_null() {
        return t.skip("string table not loaded");
    }
    let all = regions();
    let saved = capture(&all);
    for kept in [-1i32, 0, 1, 2, 3] {
        check_state(
            &mut t,
            &format!("kept {kept}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                std::ptr::write_bytes(game_addr(TEXT_RVA) as *mut u8, 0xAA, TEXT_LEN);
                game_write(KEPT_RVA, kept);
            },
            &all,
            &rust,
        );
    }
    restore(&all, &saved);
    t.finish()
}
