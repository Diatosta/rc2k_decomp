//! `screens::savegame_page_numbers` (0x46D262): the numbers of the saved games' list
//! for the page shown.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! With n = the page (`dword_75523C`) times 10: the caption (`g_captionText`) is n and
//! n + 9 with a dash between, and the ten texts of four bytes from `byte_A8C158` on
//! are n .. n + 9; each number at least two digits (a 0 before one under 10), each
//! text ended by a NUL. In four draw lists (the load and save screens).
//! Refuses a page whose numbers would not fit their four bytes (100 or more).
//! Self-test: twelve pages.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::menu::name_lists::SAVEGAME_PAGE_RVA;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use crate::strings::append_u32::append_u32;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook = RawHook::new(
    "screens::savegame_page_numbers",
    rva(0x46D262),
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

// ---- game data ----

const CAPTION_RVA: usize = rva(0xA8BE54); // g_captionText
const NUMBERS_RVA: usize = rva(0xA8C158); // byte_A8C158: ten texts of four bytes

// ---- Rust version ----

/// A number with a 0 before it under 10.
fn two_digits(n: u32) -> Vec<u8> {
    let mut text = if n < 10 { append_u32(0) } else { Vec::new() };
    text.extend(append_u32(n));
    text
}

pub fn savegame_page_numbers() -> Result<(), Refused> {
    let first = unsafe { game_read::<u32>(SAVEGAME_PAGE_RVA) }.wrapping_mul(10);
    // Signed in the original's comparisons: a negative number is "under 10" too.
    if (first as i32) < 0 || first.wrapping_add(9) >= 1000 {
        return Err(BadInput("a page whose numbers do not fit their places").into());
    }
    let numbers: Vec<Vec<u8>> = (0..10).map(|n| two_digits(first + n)).collect();
    if numbers.iter().any(|text| text.len() > 3) {
        return Err(BadInput("a number longer than its four bytes").into());
    }
    let mut caption = two_digits(first);
    caption.push(b'-');
    caption.extend(two_digits(first + 9));
    caption.push(0);
    unsafe {
        std::ptr::copy_nonoverlapping(
            caption.as_ptr(),
            game_addr(CAPTION_RVA) as *mut u8,
            caption.len(),
        );
        for (n, text) in numbers.iter().enumerate() {
            let place = game_addr(NUMBERS_RVA) as *mut u8;
            std::ptr::copy_nonoverlapping(text.as_ptr(), place.add(n * 4), text.len());
            *place.add(n * 4 + text.len()) = 0;
        }
    }
    Ok(())
}

// ---- hook ----

fn regions() -> [Region; 2] {
    [
        (game_addr(CAPTION_RVA) as u32, 0x10),
        (game_addr(NUMBERS_RVA) as u32, 40),
    ]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    savegame_page_numbers()?;
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
    let inputs = [at(SAVEGAME_PAGE_RVA)];
    let saved = (capture(&regions()), capture(&inputs));
    for page in [0u32, 1, 2, 5, 9, 10, 11, 25, 89, 98, 99, 3] {
        check_state(
            &mut t,
            &format!("page={page}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                game_write(SAVEGAME_PAGE_RVA, page);
                restore(&regions(), &[0xAAu8; 0x10 + 40]);
            },
            &regions(),
            &rust,
        );
    }
    restore(&regions(), &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
