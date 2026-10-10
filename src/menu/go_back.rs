//! `menu::go_back` (0x471C82): sets three variables of the menu system: `dword_75555D` = 0,
//! `dword_755561` = 1, `dword_755A0C` = -1.
//!
//! In:  nothing
//! Out: every register preserved
//! It is the action of the back button of several menus and the last step of other actions
//! (seven callers, ten menu tables), so it is read as "leave this menu"; what each of the
//! three variables means has not been worked out.

use crate::errors::Refused;
use crate::framework::game_write;
use crate::framework::{RawHook, Regs, SelfTest, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::go_back", rva(0x471C82), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const A_RVA: usize = rva(0x75555D); // dword_75555D
const B_RVA: usize = rva(0x755561); // dword_755561
const C_RVA: usize = rva(0x755A0C); // dword_755A0C

pub fn regions() -> [Region; 3] {
    [at(A_RVA), at(B_RVA), at(C_RVA)]
}

// ---- Rust version ----

pub fn go_back() {
    unsafe {
        game_write(A_RVA, 0u32);
        game_write(B_RVA, 1u32);
        game_write(C_RVA, u32::MAX);
    }
}

// ---- hook ----

fn rust(_: &Regs) -> Result<Left, Refused> {
    go_back();
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let saved = capture(&regions());
    for fill in [0u32, 0x5EED_0001] {
        check_state(
            &mut t,
            &format!("fill={fill:#x}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                game_write(A_RVA, fill);
                game_write(B_RVA, fill);
                game_write(C_RVA, fill);
            },
            &regions(),
            &rust,
        );
    }
    restore(&regions(), &saved);
    t.finish()
}
