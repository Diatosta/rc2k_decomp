//! The start's "go". With the grades' mode 1, the start count, `go_stand_timer` and `finish_flag`
//! all 0, and cheat bit 16 clear: `set_flag` on `StageSounds.once[2]`, then `go_stand_timer` = 1,
//! `notice_id` 0xC7, `notice_time_b` 0x32 and `notice_time` -1.
//!
//! Refuses what `set_flag` refuses.

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::physics::listed_car_flags::set_flag;
use crate::race::frame_store::FrameWork;
use crate::race::work_store::RaceWork;
use rc2k_core::physics::progress::{GO_NOTICE, GoFacts};

pub(super) fn start_go_notice(
    frame_work: &FrameWork,
    work: &RaceWork,
    car: &Car,
    sound: &crate::sound::owner::Sound,
    cheats: u32,
) -> Result<(), Refused> {
    let c = car;
    let facts = GoFacts {
        mode: c.driver_kind.get(),
        countdown: c.time_left.get(),
        gone: c.go_stand_timer.get(),
        finished: c.finish_flag.get(),
        cheats,
    };
    if !facts.go_due() {
        return Ok(());
    }
    set_flag(frame_work, work, &sound.race.stage.get().once[2])?;
    c.go_stand_timer.set(1);
    c.notice_id.set(GO_NOTICE);
    c.notice_time_b.set(0x32);
    c.notice_time.set(u32::MAX);
    Ok(())
}
