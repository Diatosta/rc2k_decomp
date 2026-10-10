//! `menu::handle_yes_no` (0x470D2E): on a screen that asks a yes/no question, lets a key
//! answer it.
//!
//! In:  eax = the key
//! Out: every register preserved
//! Only while `dword_7554CC` is 1. A key that `input::yes_no_for_key` takes for yes stores 1
//! in `dword_75524C`, one for no stores 0; either also sets `dword_758806` to 1 and leaves
//! the screen with `menu::go_back`. One caller, `menu::handle_keys`.
//! Refuses what the yes/no functions refuse.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_read, game_write, hook_asm, rva};
use crate::input::yes_no_for_key::yes_no_for_key;
use crate::input::yes_no_key::{NO, YES, yes_no_regions, yes_no_unavailable};
use crate::menu::go_back::go_back;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::handle_yes_no", rva(0x470D2E), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

pub const ASKING_RVA: usize = rva(0x7554CC); // dword_7554CC: 1 on a yes/no screen
const ANSWER_RVA: usize = rva(0x75524C); // dword_75524C: 1 = yes, 0 = no
pub const LEAVING_RVA: usize = rva(0x758806); // dword_758806

/// What the function may write.
pub fn handle_yes_no_regions() -> Vec<Region> {
    [
        &yes_no_regions()[..],
        &[at(ANSWER_RVA), at(LEAVING_RVA)],
        &crate::menu::go_back::regions(),
    ]
    .concat()
}

// ---- Rust version ----

pub unsafe fn handle_yes_no(eax: u32) -> Result<(), BadInput> {
    unsafe {
        if game_read::<u32>(ASKING_RVA) != 1 {
            return Ok(());
        }
        let answer = match yes_no_for_key(eax)? {
            YES => 1u32,
            NO => 0,
            _ => return Ok(()),
        };
        game_write(ANSWER_RVA, answer);
        game_write(LEAVING_RVA, 1u32);
    }
    go_back();
    Ok(())
}

// ---- hook ----

fn rust(r: &Regs) -> Result<Left, Refused> {
    unsafe { handle_yes_no(r.eax) }?;
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &handle_yes_no_regions(), &rust);
}

// ---- self-test: every key, on a question screen and off one ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    if let Some(why) = yes_no_unavailable() {
        return t.skip(why);
    }
    let regions = [&handle_yes_no_regions()[..], &[at(ASKING_RVA)]].concat();
    let saved = capture(&regions);
    let compared = regions.len() - 1;
    for asking in [1u32, 0, 2] {
        for key in 0..0x80u32 {
            let before = Regs {
                eax: key,
                ..Regs::SENTINEL
            };
            check_state(
                &mut t,
                &format!("asking={asking} key={key:#x}"),
                call_original,
                before,
                &|| {
                    restore(&regions, &saved);
                    unsafe {
                        game_write(ASKING_RVA, asking);
                        game_write(ANSWER_RVA, 0x5EEDu32);
                        game_write(LEAVING_RVA, 0x5EEDu32);
                    }
                },
                &regions[..compared],
                &rust,
            );
        }
    }
    restore(&regions, &saved);
    t.finish()
}
