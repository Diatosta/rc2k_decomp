//! `screens::car_page_title` (0x469A34): the car specification page's title.
//!
//! In:  nothing
//! Out: every register preserved
//!
//! The string `dword_759AFA[dword_755998]` (the page: 0x24C, 0x249, 0x24B, 0x24A) and a
//! NUL into `byte_A8BF54`. In two draw lists.
//! Refuses a page past the four, and a title longer than 0x100 bytes.
//! Self-test: the four pages.

use crate::errors::{BadInput, Refused};
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::screens::car_caption::put_string;
use crate::screens::harness::Left;
use crate::service::harness::{Region, at, capture, check_state, dispatch_state, restore};
use crate::strings::string_table::blob;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("screens::car_page_title", rva(0x469A34), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const PAGE_RVA: usize = rva(0x755998); // dword_755998
const TITLES_RVA: usize = rva(0x759AFA); // dword_759AFA
const TITLE_RVA: usize = rva(0xA8BF54); // byte_A8BF54
const TITLE_LEN: usize = 0x100;
const PAGES: u32 = 4;

// ---- Rust version ----

pub unsafe fn car_page_title() -> Result<(), Refused> {
    unsafe {
        let page: u32 = game_read(PAGE_RVA);
        if page >= PAGES {
            return Err(BadInput("a page past the four").into());
        }
        put_string(
            game_read(TITLES_RVA + page as usize * 4),
            TITLE_RVA,
            TITLE_LEN,
        )
    }
}

// ---- hook ----

fn regions() -> [Region; 1] {
    [(game_addr(TITLE_RVA) as u32, TITLE_LEN)]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { car_page_title() }?;
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
    let inputs = [at(PAGE_RVA)];
    let saved = (capture(&regions()), capture(&inputs));
    for page in 0..PAGES {
        check_state(
            &mut t,
            &format!("page={page}"),
            call_original,
            Regs::SENTINEL,
            &|| unsafe {
                game_write(PAGE_RVA, page);
                restore(&regions(), &[0xAAu8; TITLE_LEN]);
            },
            &regions(),
            &rust,
        );
    }
    restore(&regions(), &saved.0);
    restore(&inputs, &saved.1);
    t.finish()
}
