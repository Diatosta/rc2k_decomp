//! `screens::player_number_text` (0x46A359): the current player's title, as a text of the
//! car choice.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! String 0x191 + the current player's number (`dword_75598C`), a space, string 0x170, a
//! NUL, into `byte_A8C054`. In six entry and draw lists (the car choice's). Refuses a text
//! longer than its place (0x100). Self-test: the four players, the game's strings.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::take_shown_car::CURRENT_PLAYER_RVA;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use crate::strings::append_string_by_id::append_string_by_id;
use crate::strings::string_table::blob;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::player_number_text",
    rva(0x46A359),
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

const TEXT_RVA: usize = rva(0xA8C054); // byte_A8C054
const TEXT_LEN: usize = 0x100;
const FIRST_STRING: u32 = 0x191;
const SECOND_STRING: u32 = 0x170;

pub unsafe fn player_number_text() -> Result<(), Refused> {
    unsafe {
        let n: u32 = game_read(CURRENT_PLAYER_RVA);
        let mut text = Vec::new();
        let mut emit =
            |p: *const u8, len: usize| text.extend_from_slice(std::slice::from_raw_parts(p, len));
        append_string_by_id(FIRST_STRING.wrapping_add(n), &mut emit);
        text.push(b' ');
        let mut emit =
            |p: *const u8, len: usize| text.extend_from_slice(std::slice::from_raw_parts(p, len));
        append_string_by_id(SECOND_STRING, &mut emit);
        if text.len() >= TEXT_LEN {
            return Err(BadInput("a text longer than its place").into());
        }
        text.push(0);
        std::ptr::copy_nonoverlapping(text.as_ptr(), game_addr(TEXT_RVA) as *mut u8, text.len());
    }
    Ok(())
}

fn regions() -> [Region; 1] {
    [(game_addr(TEXT_RVA) as u32, TEXT_LEN)]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { player_number_text() }?;
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
    let mut all = regions().to_vec();
    all.push(at(CURRENT_PLAYER_RVA));
    let saved = capture(&all);
    for n in 0..4u32 {
        check_state(
            &mut t,
            &format!("player {n}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                restore(&all, &saved);
                std::ptr::write_bytes(game_addr(TEXT_RVA) as *mut u8, 0xAA, TEXT_LEN);
                game_write(CURRENT_PLAYER_RVA, n);
            },
            &regions(),
            &rust,
        );
    }
    restore(&all, &saved);
    t.finish()
}
