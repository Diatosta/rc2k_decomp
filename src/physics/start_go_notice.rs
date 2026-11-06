//! `physics::start_go_notice` (0x436A4B): the start's "go". With the grades' mode
//! (`g_car` +0x25C) 1, the count +0x8C, +0xA4 (`unk_71A07C`) and +0xA8 all 0, and cheat
//! bit 16 of `g_cheats` clear (`bt`): `physics::set_flag_5165a4`, then +0xA4 1, the
//! notice +0x71C 0xC7, +0x730 0x32 and +0x72C -1.
//!
//! In:  nothing
//! Out: when it ran, eax as `set_flag_5165a4` leaves it; the rest kept; no flags read
//!      (the caller's next is a call)
//!
//! Caller: `sub_436790` (0x4367E3). Refuses what `set_flag_5165a4` refuses.

use crate::errors::Refused;
use crate::physics::car_store::CAR;
use crate::physics::listed_car_flags::set_flag;
use crate::race::owner_frame::Frame;
use rc2k_core::physics::progress::{GO_NOTICE, GoFacts};

/// The Rust version.
pub fn start_go_notice(frame: &Frame) -> Result<(), Refused> {
    let c = &CAR;
    let facts = GoFacts {
        mode: c.driver_kind.get(),
        countdown: c.time_left.get(),
        gone: c.go_stand_timer.get(),
        finished: c.finish_flag.get(),
        cheats: crate::menu::owner::menus().play.cheats.cheats.get(),
    };
    if !facts.go_due() {
        return Ok(());
    }
    set_flag(frame, &crate::sound::owner::sound().race.stage.once[2])?;
    c.go_stand_timer.set(1);
    c.notice_id.set(GO_NOTICE);
    c.notice_time_b.set(0x32);
    c.notice_time.set(u32::MAX);
    Ok(())
}
