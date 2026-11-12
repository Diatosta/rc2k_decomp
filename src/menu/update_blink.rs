//! `menu::update_blink` (0x47E826): once a frame, works out from the clock what blinks and
//! whether the game has been left alone for a long time.
//!
//! After `update_frame_clock`:
//! - `draw_work.blink` (the highlight's blink, as the service area's markers use it) is 1 while
//!   bit 8 of `clock - champ_view.last_key_time` is clear, so it is on for 256 ms and off for 256 ms,
//!   counted from the last key; it stays 0 while `option_work.redefining_key` is set;
//! - `draw_work.cursor_on` (the text entry's cursor) is 1 while bit 9 of
//!   `clock - champ_view.cursor_time` is
//!   clear: 512 ms on, 512 ms off;
//! - `loop_flags.idle` is 1 once `clock - champ_view.last_key_time` has reached 300000: five minutes
//!   without a key.
//!
//! 179 callers.
use crate::menu::owner::Menus;
use crate::menu::update_frame_clock::update_frame_clock;
use crate::system::owner::System;

// ---- game data ----

// The logic is in `rc2k_core::menu::blink::blink_states`.
pub use rc2k_core::menu::blink::blink_states;

/// What the three variables should be for the clock the game holds now.
fn states_now(menus: &Menus, system: &System) -> (bool, bool, bool) {
    blink_states(
        system.session.clocks.frame.get(),
        menus.play.champ_view.last_key_time.get(),
        menus.play.champ_view.cursor_time.get(),
        menus.screens.option_work.redefining_key.is_set(),
    )
}

pub fn update_blink(menus: &Menus, system: &System) {
    let screens = &menus.screens;
    update_frame_clock(system);
    {
        let (blink, cursor_on, idle) = states_now(menus, system);
        screens.draw_work.blink.set(blink);
        screens.draw_work.cursor_on.set(cursor_on);
        screens.loop_flags.idle.set(idle);
    }
}
