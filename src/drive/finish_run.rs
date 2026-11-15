//! The championship's finish routine played. The car is lifted (`car_lift` x 65536 added to its
//! height); the clock restarted into `clock_a`; 0x20 dwords at `marks` and `play_end` -1; the
//! start `start_clock`. Each frame: the lead car's revs 0, `input::update`, the time
//! (`drive::finish_timing`); until the time reaches the length (`finish_lengths[finished]`) or
//! Esc or key 0x1C is down: the car's view 6 (+0x684), the first car loaded
//! (`race::load_first_car`), the camera (`drive::finish_camera`, `physics::camera_frame`), the
//! car stored (`drive::store_first_car`), the race's frame (`drive::race_frame`).
//!
//! Refuses a length index past 1 (the table's two), and what its calls refuse.

use crate::drive::race_parts::{input_update, race_frame, restart_clock};
use crate::errors::{BadInput, Refused};
use crate::math::x87_float::int;
use crate::math::x87_float::ld;
use crate::math::x87_float::to_int;
use crate::physics::flow::c0;
use crate::race::owner::Race;

/// The car's lift: `CarBlocks.cars[0].position[1]` += `car_lift` x `k65536`.
fn lift_car(race: &Race) {
    race.car_blocks().cars[0].position[1].set(to_int(
        ld(race.replay.finish.get().car_lift.get()) * ld(race.course.numbers.get().k65536.get())
            + int(race.car_blocks().cars[0].position[1].get() as i32),
    ) as u32);
}

pub fn finish_run(race: &Race, o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let (scene, sound, input) = (o.scene, o.sound, o.input);
    lift_car(race);
    restart_clock(race, o.system);
    race.frame
        .clock_ticks()
        .clock_a
        .set(race.shared.physics_tables.get().scene_clock.get());
    race.frame
        .clock_ticks()
        .sound_tick_last
        .set(race.shared.physics_tables.get().scene_clock.get());
    for mark in &race.frame.clock_ticks().marks {
        mark.set(u32::MAX);
    }
    race.replay.replay_view().play_end.set(u32::MAX);
    restart_clock(race, o.system);
    race.replay
        .finish
        .get()
        .start_clock
        .set(race.shared.physics_tables.get().scene_clock.get());
    loop {
        sound.race.sounds.get().objects[0].revs.set(0);
        input_update(o);
        crate::drive::finish_timing::finish_timing(race, scene, o.system);
        let n = race.shared.state.results.finished.get();
        if n > 1 {
            return Err(BadInput("a finish length past the table's two").into());
        }
        if !c0(
            race.replay.finish.get().time.get(),
            race.replay.finish.get().finish_lengths[n as usize].get(),
        ) {
            break;
        }
        let pressed = |key: usize| input.devices.keys_pressed.get(key) != 0;
        if pressed(1) || pressed(0x1C) {
            break;
        }
        race.car_blocks().cars[0].camera_view.set(6);
        crate::race::frame_views::load_first_car(&race.shared)?;
        crate::drive::finish_camera::finish_camera(race.frame.view_and_records.get(), &race.replay);
        crate::physics::camera_frame::camera_frame(
            &crate::physics::step_inputs::StepInputs::of(race).camera(),
            scene,
        )?;
        crate::drive::store_first_car::store_first_car(&race.shared)?;
        race_frame(race, o)?;
    }
    Ok(())
}

#[cfg(test)]
mod reference_vectors {
    use super::*;
    use crate::test_support::own::race;
    use crate::test_support::vectors::block_vectors::{Block, In, Operand, Out, sparse};

    #[test]
    fn finish_run_matches_the_assembly() {
        let blocks = vec![Block {
            name: "finish_run lift",
            operands: vec![Operand::RaceBlock {
                reads: &[(0x24BB38, In::F), (0x1C5D98, In::F), (0x30F524, In::I)],
                writes: &[(0x30F524, Out::I)],
            }],
            ulps: 0,
            plain: |_| lift_car(race()),
        }];
        sparse("drive.finish_run", &blocks, 4000, 204);
    }
}
