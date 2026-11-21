//! The work car's pace notes, each physics step. Only with `finish_flag`, `go_stand_timer`,
//! `started_stand_timer`, `hold_car_revs`, `in_credits`, `false_start_count` and the car's
//! `all_wheels_off` all 0, `driver_kind` 1 and `run_ticks` 1000 or more (unsigned). Then the
//! checkpoints ahead: the car speed x `pace_scale` / eight, rounded; the target is the car's
//! checkpoint (`reset_checkpoint`) plus that when not negative, 0 when at or past the count. Past
//! the last called (`last_pace_call`, unsigned): more than 16 past moves it up to the target first,
//! then every checkpoint after it up to the target (at least one) has its group called
//! (`physics::queue_pace_note`), and `last_pace_call` becomes the target. Then (unless it left at
//! the start) `physics::expire_car_events`.
//!
//! Refuses what its calls refuse.

use crate::errors::Refused;
use crate::physics::car_store::Car;
use crate::physics::figures::PACE_SCALE;
use crate::physics::step_families::CheckpointInputs;
use crate::race::replay_file_store::ReplayFileBlock;
use crate::race::work_store::RaceWork;
use rc2k_core::physics::pace_calls::{call_target, calling, calls};

/// The gates' variables (each must be 0; the last is "1000 or more").
fn gates(work: &RaceWork, replay_file: &ReplayFileBlock, car: &Car) -> [u32; 7] {
    [
        car.finish_flag.get(),
        car.go_stand_timer.get(),
        car.started_stand_timer.get(),
        replay_file.hold_car_revs.get(),
        u32::from(work.in_credits.is_set()),
        car.false_start_count.get(),
        car.all_wheels_off.get(),
    ]
}
const ONE_F32: u32 = 0x3F80_0000;

// Core: `rc2k_core::physics::pace_calls::{calling, call_target, calls}`.
pub fn pace_notes_step(
    c: &CheckpointInputs<'_>,
    sound: &crate::sound::owner::Sound,
    audio: &crate::audio::owner::Audio,
) -> Result<(), Refused> {
    let (physics, numbers, frame_work) = (c.physics, c.numbers, &c.frame_work);
    let (work, vars, replay_file) = (c.work, c.vars, c.shared.replay_file());
    let car = c.car;
    let scene_clock = c.shared.run().scene_clock;
    if !calling(
        gates(work, replay_file, car),
        car.driver_kind.get(),
        car.run_ticks.get(),
    ) {
        return Ok(());
    }
    // Multiply, divide, round (the extra division by 1.0 is exact).
    let ahead = crate::math::x87::mul_div_div_to_int(
        &car.local_velocity[2],
        PACE_SCALE,
        &numbers.eight,
        ONE_F32,
    );
    let target = call_target(
        car.reset_checkpoint.get(),
        ahead,
        vars.checkpoint_count.get(),
    );
    if let Some((last, _)) = calls(target, car.last_pace_call.get()) {
        car.last_pace_call.set(last);
        let mut checkpoint = last;
        loop {
            checkpoint = checkpoint.wrapping_add(1);
            crate::physics::queue_pace_note::queue_pace_note(
                physics,
                frame_work,
                work,
                scene_clock,
                car,
                checkpoint,
                sound,
                audio,
            )?;
            if checkpoint >= target {
                break;
            }
        }
        car.last_pace_call.set(target);
    }
    crate::physics::expire_car_events::expire_car_events(car, scene_clock)?;
    Ok(())
}
