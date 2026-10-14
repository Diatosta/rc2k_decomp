//! `screens::slideshow_title` (0x469A57): the slideshow's title.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! String 0x33D and a NUL into `byte_A8BF54`. In two draw lists (the slideshow).
//! Refuses a title longer than 0x100 bytes.
//! Self-test: the game's string.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, hook_asm, rva};
use crate::screens::car_caption::put_string;
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use crate::strings::string_table::blob;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("screens::slideshow_title", rva(0x469A57), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

const TITLE_RVA: usize = rva(0xA8BF54); // byte_A8BF54
const TITLE_LEN: usize = 0x100;
const TITLE: u32 = 0x33D;

// ---- Rust version ----

pub unsafe fn slideshow_title() -> Result<(), Refused> {
    unsafe { put_string(TITLE, TITLE_RVA, TITLE_LEN) }
}

// ---- hook ----

fn regions() -> [Region; 1] {
    [(game_addr(TITLE_RVA) as u32, TITLE_LEN)]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { slideshow_title() }?;
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
    if blob().is_null() {
        return t.skip("string table not loaded");
    }
    let saved = capture(&regions());
    check_state(
        &mut t,
        "the title",
        call_original,
        Regs::SENTINEL,
        &|| restore(&regions(), &[0xAAu8; TITLE_LEN]),
        &regions(),
        &rust,
    );
    restore(&regions(), &saved);
    t.finish()
}
