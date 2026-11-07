//! `drive::finish_run` (0x459360): the championship's finish routine played. The car
//! lifted (+0x138 + `flt_64BB38` x 65536, `fiadd` / `fistp`); the clock into
//! `dword_5CC3B4`, `5CC3B0`; 0x20 dwords at `unk_5CC1B0` and `dword_5C2538` -1; the start
//! `dword_64BB60`. Each frame: `dword_512A44` 0, `input::update`, the time
//! (`drive::finish_timing`); until the time reaches the length (`flt_64BB3C[n]`, n
//! `dword_75B108`) or Esc or key 0x1C is down: the car's view 6 (+0x684), the first car
//! loaded (`race::load_first_car`), the camera (`drive::finish_camera`,
//! `physics::camera_frame`), the car stored (`drive::store_first_car`), the race's frame
//! (`sub_4541BF`, stream A's: through the game).
//!
//! Caller: `drive::finish_race`. Refuses a length index past 1 (the table's two), and what
//! its calls refuse.

use crate::drive::race_parts::{input_update, race_frame, restart_clock};
use crate::errors::{BadInput, Refused};
use crate::math::x87_float::int;
use crate::math::x87_float::ld;
use crate::math::x87_float::to_int;
use crate::physics::flow::c0;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::owner::Race;

/// The car's lift: `dword_70F524` += `flt_64BB38` x `flt_5C5D98` (`fiadd`, `fistp`), over the
/// race block at `rc`.
fn lift_car(race: &Race) {
    CAR_BLOCKS.cars[0].position[1].set_bits(to_int(
        ld(race.replay.finish.unknown_20.bits()) * ld(race.course.numbers.k65536.bits())
            + int(CAR_BLOCKS.cars[0].position[1].bits() as i32),
    ) as u32);
}

/// The Rust version. The registers it leaves are read by no caller (liveness: esi after
/// `drive::finish_race`'s call has no read point).
pub fn finish_run(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    sound: &crate::sound::owner::Sound,
    input: &crate::input::owner::Input,
) -> Result<(), Refused> {
    lift_car(race);
    restart_clock(race);
    race.frame
        .clock_ticks
        .clock_a
        .set(race.shared.physics_tables.scene_clock.get());
    race.frame
        .clock_ticks
        .sound_tick_last
        .set(race.shared.physics_tables.scene_clock.get());
    for mark in &race.frame.clock_ticks.marks {
        mark.set(u32::MAX);
    }
    race.replay.replay_view.play_end.set(u32::MAX);
    restart_clock(race);
    race.replay
        .finish
        .start_clock
        .set(race.shared.physics_tables.scene_clock.get());
    loop {
        sound.race.sounds.objects[0].revs.set(0);
        input_update();
        crate::drive::finish_timing::finish_timing(race, scene);
        let n = race.shared.state.results.finished.get();
        if n > 1 {
            return Err(BadInput("a finish length past the table's two").into());
        }
        if !c0(
            race.replay.finish.time.bits(),
            race.replay.finish.finish_lengths[n as usize].bits(),
        ) {
            break;
        }
        let pressed = |key: usize| input.devices.keys_pressed.get(key) != 0;
        if pressed(1) || pressed(0x1C) {
            break;
        }
        CAR_BLOCKS.cars[0].camera_view.set_bits(6);
        crate::race::frame_views::load_first_car(&race.shared)?;
        crate::drive::finish_camera::finish_camera(&race.replay);
        crate::physics::camera_frame::camera_frame(race, scene)?;
        crate::drive::store_first_car::store_first_car(&race.shared)?;
        race_frame(race)?;
    }
    Ok(())
}

#[cfg(all(test, target_arch = "x86"))]
mod asm_reference {
    use super::*;
    use crate::race::owner::race;
    use crate::scene::asm_test::{Block, In, Operand, Out, reference, sparse};

    // The original's instructions.
    reference!(ref_lift(rc)
        "fld dword ptr [{rc} + 0x24BB38]",
        "fmul dword ptr [{rc} + 0x1C5D98]",
        "fiadd dword ptr [{rc} + 0x30F524]",
        "fistp dword ptr [{rc} + 0x30F524]");

    #[test]
    fn finish_run_matches_the_assembly() {
        let blocks = vec![Block {
            name: "finish_run lift",
            operands: vec![Operand::RaceBlock {
                reads: &[(0x24BB38, In::F), (0x1C5D98, In::F), (0x30F524, In::I)],
                writes: &[(0x30F524, Out::I)],
            }],
            ulps: 0,
            reference: |a| unsafe { ref_lift(a[0]) },
            plain: |_| lift_car(race()),
        }];
        sparse(&blocks, 4000, 204);
    }
}
