//! `menu::update_blink` (0x47E826): once a frame, works out from the clock what blinks and
//! whether the game has been left alone for a long time.
//!
//! In:  nothing
//! Out: every register preserved
//! After `update_frame_clock`:
//! - `dword_7559A4` (the highlight's blink, as the service area's markers use it) is 1 while
//!   bit 8 of `clock - dword_7556C4` is clear, so it is on for 256 ms and off for 256 ms,
//!   counted from the last key; it stays 0 while `dword_7556AC` is 1;
//! - `dword_7559A8` (the text entry's cursor) is 1 while bit 9 of `clock - dword_7556C8` is
//!   clear: 512 ms on, 512 ms off;
//! - `dword_755551` is 1 once `clock - dword_7556C4` has reached 300000: five minutes
//!   without a key.
//!
//! 179 callers.

use crate::menu::champ_store::CHAMP_VIEW;
use crate::menu::draw_work_store::DRAW_WORK;
use crate::menu::loop_store::LOOP_FLAGS;
use crate::menu::option_work_store::OPTION_WORK;
use crate::menu::update_frame_clock::update_frame_clock;
use crate::session::CLOCKS;

// ---- game data ----

// ---- Rust version ----

// Converted: `rc2k_core::menu::blink::blink_states`.
pub use rc2k_core::menu::blink::blink_states;

/// What the three variables should be for the clock the game holds now.
unsafe fn states_now() -> (u32, u32, u32) {
    blink_states(
        CLOCKS.frame.get(),
        CHAMP_VIEW.last_key_time.get(),
        CHAMP_VIEW.cursor_time.get(),
        OPTION_WORK.redefining_key.get(),
    )
}

pub fn update_blink() {
    update_frame_clock();
    unsafe {
        let (blink, cursor_on, idle) = states_now();
        DRAW_WORK.blink.set(blink);
        DRAW_WORK.cursor_on.set(cursor_on);
        LOOP_FLAGS.idle.set(idle);
    }
}
