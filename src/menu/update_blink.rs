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
use crate::menu::update_frame_clock::update_frame_clock;
use crate::session::CLOCKS;

// ---- game data ----

// ---- Rust version ----

// Converted: `rc2k_core::menu::blink::blink_states`.
pub use rc2k_core::menu::blink::blink_states;

/// What the three variables should be for the clock the game holds now.
fn states_now() -> (u32, u32, u32) {
    blink_states(
        CLOCKS.frame.get(),
        crate::menu::owner::menus()
            .play
            .champ_view
            .last_key_time
            .get(),
        crate::menu::owner::menus()
            .play
            .champ_view
            .cursor_time
            .get(),
        crate::menu::owner::menus()
            .screens
            .option_work
            .redefining_key
            .get(),
    )
}

pub fn update_blink() {
    update_frame_clock();
    {
        let (blink, cursor_on, idle) = states_now();
        crate::menu::owner::menus()
            .screens
            .draw_work
            .blink
            .set(blink);
        crate::menu::owner::menus()
            .screens
            .draw_work
            .cursor_on
            .set(cursor_on);
        crate::menu::owner::menus()
            .screens
            .loop_flags
            .idle
            .set(idle);
    }
}
