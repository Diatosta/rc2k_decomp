//! `menu::update_frame_clock`: stores `tick_count` in the window handle, the clock the
//! menu code measures blinking and idle time against.

use crate::clock::tick_count::tick_count;
use crate::system::owner::System;

pub fn update_frame_clock(system: &System) -> u32 {
    let now = tick_count(&system.session.picture_clock);
    system.clocks().frame.set(now);
    now
}
