//! `menu::build_notice` (0x4165F1): for the edition whose notice kind is 2, makes one
//! copyright line of the two.
//!
//! In:  nothing
//! Out: every register preserved
//! Only when `g_noticeKind` (the byte at 0x4E07A0, from the language's `.loc` file) is 2.
//! The first line (`unk_52A263`, "Copyright (c) Magnetic Fields (Software Design) Ltd,
//! 1999. All rights reserved.") becomes "(c) 2000 Actualize Ltd and Magnetic Fields
//! (Software Design) Ltd, " and the second (`byte_52A2B1`, "Copyright (c) Actualize Ltd,
//! 1999. ..") is emptied. It is put together in the shared text buffer from pieces of the
//! two lines themselves: two characters from the second line's eleventh on, "2000",
//! fourteen from its twelfth on, " and", 0x26 from the first line's twelfth on, a space
//! and a NUL; then copied over the first line.
//! One caller, `system::main_loop`. Calling it twice builds from the changed lines, as
//! the original would.

use crate::errors::Refused;
use crate::framework::{RawHook, Regs, SelfTest, game_addr, game_read, game_write, hook_asm, rva};
use crate::screens::harness::Left;
use crate::service::harness::{Region, capture, check_state, dispatch_state, restore};
use crate::strings::text_from_ids::text;
use std::sync::atomic::AtomicUsize;

static ORIGINAL: AtomicUsize = AtomicUsize::new(0);

pub static HOOK: RawHook =
    RawHook::new("menu::build_notice", rva(0x4165F1), thunk, &ORIGINAL).self_test(self_test);

hook_asm!(
    thunk,
    call_original,
    dispatch = dispatch,
    original = ORIGINAL
);

// ---- game data ----

const NOTICE_KIND_RVA: usize = rva(0x4E07A0); // g_noticeKind
const FIRST_LINE_RVA: usize = rva(0x52A263); // unk_52A263
const SECOND_LINE_RVA: usize = rva(0x52A2B1); // byte_52A2B1
/// The whole of the two lines.
const LINES_LEN: usize = 0xA0;
use rc2k_core::menu::notice::{
    BUILDS, BUILT_LEN, FIRST_FROM, FIRST_READ, SECOND_FROM, SECOND_READ, notice,
};

// ---- Rust version ----

pub unsafe fn build_notice() {
    // Converted: `rc2k_core::menu::notice::notice`.
    unsafe {
        if game_read::<u8>(NOTICE_KIND_RVA) != BUILDS {
            return;
        }
        let built = notice(
            &game_read::<[u8; FIRST_READ]>(FIRST_LINE_RVA + FIRST_FROM),
            &game_read::<[u8; SECOND_READ]>(SECOND_LINE_RVA + SECOND_FROM),
        );
        std::ptr::copy(built.as_ptr(), text(), BUILT_LEN);
        std::ptr::copy(
            built.as_ptr(),
            game_addr(FIRST_LINE_RVA) as *mut u8,
            BUILT_LEN,
        );
        game_write(SECOND_LINE_RVA, 0u8);
    }
}

// ---- hook ----

fn regions() -> [Region; 2] {
    [
        (text() as u32, BUILT_LEN),
        (game_addr(FIRST_LINE_RVA) as u32, LINES_LEN),
    ]
}

fn rust(_: &Regs) -> Result<Left, Refused> {
    unsafe { build_notice() };
    Ok(Left::nothing())
}

extern "C" fn dispatch(r: &mut Regs) {
    dispatch_state(&HOOK, r, call_original, &regions(), &rust);
}

// ---- self-test: the game's two lines and lines of the test's, with each notice kind ----

fn self_test() -> (usize, usize) {
    let Some(mut t) = SelfTest::begin(&HOOK) else {
        return (0, 0);
    };
    let regions = regions();
    let kind = [(game_addr(NOTICE_KIND_RVA) as u32, 1usize)];
    let saved = (capture(&regions), capture(&kind));
    let other: Vec<u8> = (0..LINES_LEN).map(|n| b'a' + (n % 26) as u8).collect();
    for (what, lines) in [("the game's lines", None), ("other lines", Some(&other))] {
        for notice_kind in [0u8, 1, 2, 3, 7] {
            check_state(
                &mut t,
                &format!("{what}, notice kind {notice_kind}"),
                call_original,
                Regs::SENTINEL,
                &|| unsafe {
                    restore(&regions, &saved.0);
                    text().write_bytes(0xAA, BUILT_LEN);
                    if let Some(lines) = lines {
                        std::ptr::copy(
                            lines.as_ptr(),
                            game_addr(FIRST_LINE_RVA) as *mut u8,
                            LINES_LEN,
                        );
                    }
                    game_write(NOTICE_KIND_RVA, notice_kind);
                },
                &regions,
                &rust,
            );
        }
    }
    restore(&regions, &saved.0);
    restore(&kind, &saved.1);
    t.finish()
}
