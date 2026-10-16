//! `menu::next_gearing` (0x469948): the car choice's gearing on (past 2 to 0).
//!
//! In:  nothing
//! Out: every register preserved
//!
//! See `screens/gearing_text.rs`: `dword_7554F0`, the current player's `gearing` (+0x30)
//! and its text (string 0x21F + it into `byte_A8BF54`). Its item in the car choice's
//! table (0x74B978). Refuses a current player past four. Self-test: the game's player and
//! strings, `dword_7554F0` from -1 to 3, put back afterwards.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_write, hook_asm, rva};
use crate::menu::player::Player;
use crate::screens::gearing_text::{KEPT_RVA, TEXT_LEN, TEXT_RVA, current, regions, step_gearing};
use crate::screens::harness::Left;
use crate::service::harness::{capture, check_state, dispatch_state, restore};
use crate::strings::string_table::blob;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::next_gearing", rva(0x469948), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { step_gearing(true) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    let mut regions = regions().to_vec();
    if let Ok(p) = current() {
        regions.push((p, std::mem::size_of::<Player>()));
    }
    dispatch_state(&HOOK, r, call_original, &regions, &rust);
}

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if blob().is_null() {
        return t.skip("string table not loaded");
    }
    let Ok(p) = current() else {
        return t.skip("no current player");
    };
    let mut all = regions().to_vec();
    all.push((p, std::mem::size_of::<Player>()));
    let saved = capture(&all);
    for kept in [-1i32, 0, 1, 2, 3] {
        check_state(
            &mut t,
            &format!("kept {kept}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&all, &saved);
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
