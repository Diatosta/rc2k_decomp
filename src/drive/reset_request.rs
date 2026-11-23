//! The reset key held. Once the race is past 0x320
//! (`run_ticks`, by the sign): with the keys' bit 0x80 (`input_keys`), outside mode 2
//! (`run_kind`) and the car's speed below `reset_speed_limit` (or unordered): a notice set up on the
//! first step (`notice_id` 0xCD, `notice_time_b` 0x19, `notice_time` 0x12C), the count `reset_key_count` +
//! 1, at 100 the time penalty (0x2BC less `stuck_count`) added to `stage_time`, the notice cleared
//! and `physics::reset_car`. Else a count under way is cleared with the notice.
//!
//! Refuses what `physics::reset_car` refuses.

use crate::errors::Refused;
use crate::physics::figures::RESET_SPEED_LIMIT;
use crate::physics::flow::c0;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use rc2k_core::drive::controls::{
    RESET_NOTICE, reset_counts, reset_held, reset_penalty, reset_step,
};

fn clear_notice(shared: &RaceShared) {
    let mut car = shared.car_mut();
    car.notice_last = 0;
    car.notice_id = 0;
    car.notice_time = 0;
    car.notice_shown = 0;
}

/// Then `physics::reset_car`.
pub fn reset_request(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let race_owner = race;

    if !reset_counts(race_owner.shared.car().run_ticks) {
        return Ok(());
    }
    if !reset_held(
        race_owner.shared.car().input_keys,
        race_owner.shared.vars.get().run_kind.get(),
        || c0(race_owner.shared.car().speed, RESET_SPEED_LIMIT),
    ) {
        if race_owner.shared.car().reset_key_count != 0 {
            clear_notice(&race.shared);
            race_owner.shared.car_mut().reset_key_count = 0;
        }
        return Ok(());
    }
    let (first, count, reset) = reset_step(race_owner.shared.car().reset_key_count);
    if first {
        race_owner.shared.car_mut().notice_id = RESET_NOTICE;
        race_owner.shared.car_mut().notice_time_b = 0x19;
        race_owner.shared.car_mut().notice_time = 0x12C;
    }
    race_owner.shared.car_mut().reset_key_count = count;
    if !reset {
        return Ok(());
    }
    let value = race_owner
        .shared
        .car()
        .stage_time
        .wrapping_add(reset_penalty(race_owner.shared.car().stuck_count));
    race_owner.shared.car_mut().stage_time = value;
    clear_notice(&race.shared);
    {
        crate::physics::reset_car::reset_car(
            &crate::physics::step_inputs::StepInputs::of(race).reset(),
            scene,
        )
    }?;
    Ok(())
}
