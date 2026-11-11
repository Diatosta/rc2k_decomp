//! `physics::start_go_notice` (0x436A4B): the start's "go". With the grades' mode
//! (`g_car` +0x25C) 1, the count +0x8C, +0xA4 (`go_stand_timer`) and +0xA8 all 0, and cheat
//! bit 16 of `g_cheats` clear (`bt`): `physics::set_flag_5165a4`, then +0xA4 1, the
//! notice +0x71C 0xC7, +0x730 0x32 and +0x72C -1.
//!
//! Caller: `physics::race_timers` (0x4367E3). Refuses what `set_flag_5165a4` refuses.

use crate::errors::Refused;
use crate::menu::owner::menus;
use crate::physics::car_store::Car;
use crate::physics::listed_car_flags::set_flag;
use crate::race::frame_store::FrameWork;
use crate::race::work_store::RaceWork;
use rc2k_core::physics::progress::{GO_NOTICE, GoFacts};

pub fn start_go_notice(
    frame_work: &FrameWork,
    work: &RaceWork,
    car: &Car,
    sound: &crate::sound::owner::Sound,
) -> Result<(), Refused> {
    let c = car;
    let facts = GoFacts {
        mode: c.driver_kind.get(),
        countdown: c.time_left.get(),
        gone: c.go_stand_timer.get(),
        finished: c.finish_flag.get(),
        cheats: menus().play.cheats.cheats.get(),
    };
    if !facts.go_due() {
        return Ok(());
    }
    set_flag(frame_work, work, &sound.race.stage.once[2])?;
    c.go_stand_timer.set(1);
    c.notice_id.set(GO_NOTICE);
    c.notice_time_b.set(0x32);
    c.notice_time.set(u32::MAX);
    Ok(())
}
