//! `drive::reset_request` (0x43A0D3): the reset key held. Once the race is past 0x320
//! (`run_ticks`, by the sign): with the keys' bit 0x80 (`input_keys`), outside mode 2
//! (`run_kind`) and the car's speed below `reset_speed_limit` (or unordered): a notice set up on the
//! first step (`notice_id` 0xCD, `notice_time_b` 0x19, `notice_time` 0x12C), the count `reset_key_count` +
//! 1, at 100 the time penalty (0x2BC less `stuck_count`) added to `stage_time`, the notice cleared
//! and `physics::reset_car`. Else a count under way is cleared with the notice.
//!
//! Callers: `drive::car_controls`, `race::step_replay_cars`, `drive::replay_step`. Refuses what
//! `physics::reset_car` refuses.

use crate::errors::Refused;
use crate::physics::flow::c0;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use rc2k_core::drive::controls::{
    RESET_NOTICE, reset_counts, reset_held, reset_penalty, reset_step,
};

fn clear_notice(shared: &RaceShared) {
    for cell in [
        &shared.car().notice_last,
        &shared.car().notice_id,
        &shared.car().notice_time,
        &shared.car().notice_shown,
    ] {
        cell.set(0);
    }
}

/// Then `physics::reset_car`.
// Converted: `rc2k_core::drive::controls::{reset_counts, reset_held, reset_step, reset_penalty}`.
pub fn reset_request(race: &Race, scene: &crate::scene::owner::Scene) -> Result<(), Refused> {
    let race_owner = race;

    if !reset_counts(race_owner.shared.car().run_ticks.get()) {
        return Ok(());
    }
    if !reset_held(
        race_owner.shared.car().input_keys.get(),
        race_owner.shared.vars.run_kind.get(),
        || {
            c0(
                race_owner.shared.car().speed.get(),
                race_owner.shared.physics_tables.reset_speed_limit.get(),
            )
        },
    ) {
        if race_owner.shared.car().reset_key_count.get() != 0 {
            clear_notice(&race.shared);
            race_owner.shared.car().reset_key_count.set(0);
        }
        return Ok(());
    }
    let (first, count, reset) = reset_step(race_owner.shared.car().reset_key_count.get());
    if first {
        race_owner.shared.car().notice_id.set(RESET_NOTICE);
        race_owner.shared.car().notice_time_b.set(0x19);
        race_owner.shared.car().notice_time.set(0x12C);
    }
    race_owner.shared.car().reset_key_count.set(count);
    if !reset {
        return Ok(());
    }
    race_owner.shared.car().stage_time.set(
        race_owner
            .shared
            .car()
            .stage_time
            .get()
            .wrapping_add(reset_penalty(race_owner.shared.car().stuck_count.get())),
    );
    clear_notice(&race.shared);
    {
        crate::physics::reset_car::reset_car(
            &crate::physics::step_inputs::StepInputs::of(race).reset(),
            scene,
        )
    }?;
    Ok(())
}
