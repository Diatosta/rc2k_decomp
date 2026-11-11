//! `physics::pace_notes_step` (0x4417EC): the work car's pace notes, each physics step. Only
//! with `finish_flag`, `go_stand_timer`, `started_stand_timer`, `g_holdCarRevs`, `in_credits`,
//! `false_start_count` and the car's `all_wheels_off` all 0, `driver_kind` 1 and `run_ticks` 1000 or
//! more (unsigned). Then the checkpoints ahead: `g_carSpeed` x `pace_scale` / eight, rounded;
//! the target is the car's checkpoint (`reset_checkpoint`) plus that when not negative, 0 when
//! at or past the count (by the sign). Past the last called (`last_pace_call`, unsigned): more than
//! 16 past moves it up to the target first, then every checkpoint after it up to the target (at
//! least one) has its group called (`physics::queue_pace_note`), and `last_pace_call` becomes the
//! target. Then (unless it left at the start) `physics::expire_car_events`.
//!
//! Caller: `physics::car_physics_step`. Its calls by `physics/queue_pace_note.rs`' places
//! (5 the group, 6 the expiry). Refuses what its calls refuse.

use crate::errors::Refused;
use crate::race::owner::Race;
use rc2k_core::physics::pace_calls::{call_target, calling, calls};

/// The gates' variables (each must be 0; the last is "1000 or more").
fn gates(race: &Race) -> [u32; 7] {
    [
        race.shared.car.finish_flag.get(),
        race.shared.car.go_stand_timer.get(),
        race.shared.car.started_stand_timer.get(),
        race.shared.replay_file.hold_car_revs.get(),
        u32::from(race.shared.work.in_credits.is_set()),
        crate::race::owner::RACE.shared.car.false_start_count.get(),
        crate::race::owner::RACE.shared.car.all_wheels_off.get(),
    ]
}
const ONE_F32: u32 = 0x3F80_0000;

// Converted: `rc2k_core::physics::pace_calls::{calling, call_target, calls}`.
pub fn pace_notes_step(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    sound: &crate::sound::owner::Sound,
) -> Result<(), Refused> {
    if !calling(
        gates(race),
        race.shared.car.driver_kind.get(),
        race.shared.car.run_ticks.get(),
    ) {
        return Ok(());
    }
    // `fld; fmul; fdiv; fistp` (the extra division by 1.0 is exact).
    let ahead = crate::math::x87::mul_div_div_to_int(
        race.shared.car.local_velocity[2].bits(),
        race.shared.physics_tables.pace_scale.bits(),
        race.course.numbers.eight.bits(),
        ONE_F32,
    );
    scene.drawing.scene.scratch.set(ahead as u32);
    let target = call_target(
        race.shared.car.reset_checkpoint.get(),
        ahead,
        race.shared.vars.checkpoint_count.get(),
    );
    if let Some((last, _)) = calls(target, race.shared.car.last_pace_call.get()) {
        race.shared.car.last_pace_call.set(last);
        let mut checkpoint = last;
        loop {
            checkpoint = checkpoint.wrapping_add(1);
            crate::physics::queue_pace_note::queue_pace_note(race, checkpoint, sound)?;
            if checkpoint >= target {
                break;
            }
        }
        race.shared.car.last_pace_call.set(target);
    }
    crate::physics::expire_car_events::expire_car_events(
        &race.shared.car,
        &race.shared.physics_tables,
    )?;
    Ok(())
}
