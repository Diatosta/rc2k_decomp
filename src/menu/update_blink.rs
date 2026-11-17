//! `menu::update_blink`: once a frame, works out from the clock what blinks and whether the game
//! has been left alone for a long time.
//!
//! After `update_frame_clock`:
//! - `draw_work.blink` (the highlight's blink, as the service area's markers use it) is 1 while bit
//!   8 of `clock - champ_view.key_clock.last_key_time` is clear, so it is on for 256 ms and off
//!   for 256 ms, counted from the last key; it stays 0 while `entry.redefining_key` is set;
//! - `draw_work.cursor_on` (the text entry's cursor) is 1 while bit 9 of
//!   `clock - champ_view.key_clock.cursor_time` is clear: 512 ms on, 512 ms off;
//! - `loop_flags.idle` is 1 once `clock - champ_view.key_clock.last_key_time` has reached
//!   300000: five minutes without a key.
use crate::menu::champ_store::ChampView;
use crate::menu::entry_store::EntryState;
use crate::menu::owner::Menus;
use crate::menu::update_frame_clock::update_frame_clock;
use crate::system::owner::System;

pub use rc2k_core::menu::blink::blink_states;

/// What the three variables should be for the clock the game holds now.
fn states_now(champ: &ChampView, entry: &EntryState, system: &System) -> (bool, bool, bool) {
    blink_states(
        system.clocks().frame.get(),
        champ.key_clock.last_key_time.get(),
        champ.key_clock.cursor_time.get(),
        entry.redefining_key.is_set(),
    )
}

pub fn update_blink(menus: &Menus, system: &System) {
    let screens = &menus.screens;
    update_frame_clock(system);
    // What other threads (the sound's) found wrong waits for a menu frame to be told.
    crate::error_dialog::show_pending();
    {
        let (blink, cursor_on, idle) = states_now(&menus.play.champ_view, &screens.entry, system);
        screens.draw_work.blink.set(blink);
        screens.draw_work.cursor_on.set(cursor_on);
        screens.loop_flags.idle.set(idle);
    }
}
