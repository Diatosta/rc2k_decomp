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
//! Hook removed 2026-10-22 (direct calls): Rust callers run its `entry` (`calls::call`,
//! `routes.rs`); the registers above are the original's, for reference.

use crate::framework::{game_addr, game_read, game_write, rva};
use crate::strings::text_from_ids::text;

// ---- game data ----

const FIRST_LINE_RVA: usize = rva(0x52A263); // unk_52A263
const SECOND_LINE_RVA: usize = rva(0x52A2B1); // byte_52A2B1
use rc2k_core::menu::notice::{
    BUILDS, BUILT_LEN, FIRST_FROM, FIRST_READ, SECOND_FROM, SECOND_READ, notice,
};

// ---- Rust version ----

pub unsafe fn build_notice() {
    // Converted: `rc2k_core::menu::notice::notice`.
    unsafe {
        if crate::strings::load_language::notice_kind() != BUILDS {
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
