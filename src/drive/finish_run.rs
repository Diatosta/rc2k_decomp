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
//! In:  nothing
//! Out: the registers as the steps leave them; no flags read
//!
//! Caller: `drive::finish_race`. Refuses a length index past 1 (the table's two), and what
//! its calls refuse.

use crate::drive::race_parts::{input_update, race_frame, restart_clock};
use crate::errors::{BadInput, Refused};
use crate::physics::flow::c0;
use crate::physics::tables_store::PHYSICS;
use crate::race::blocks_store::CAR_BLOCKS;
use crate::race::clock_ticks_store::CLOCK_TICKS;
use crate::race::state_store::RACE_STATE;
use crate::race::viewer_state_store::REPLAY_VIEW;
use crate::scene::x87_mem::{fild, fistp, fld};

/// The car's lift: `dword_70F524` += `flt_64BB38` x `flt_5C5D98` (`fiadd`, `fistp`), over the
/// race block at `rc`.
fn lift_car(rc: usize) {
    fistp(
        rc,
        0x30F524,
        fld(rc, 0x24BB38) * fld(rc, 0x1C5D98) + fild(rc, 0x30F524),
    )
}

/// The Rust version. The registers it leaves are read by no caller (liveness: esi after
/// `drive::finish_race`'s call has no read point).
pub fn finish_run() -> Result<(), Refused> {
    lift_car(crate::race::store::asm_base() as usize);
    restart_clock();
    CLOCK_TICKS.clock_a.set(PHYSICS.scene_clock.get());
    CLOCK_TICKS.sound_tick_last.set(PHYSICS.scene_clock.get());
    for mark in &CLOCK_TICKS.marks {
        mark.set(u32::MAX);
    }
    REPLAY_VIEW.play_end.set(u32::MAX);
    restart_clock();
    crate::drive::finish_store::FINISH
        .start_clock
        .set(PHYSICS.scene_clock.get());
    loop {
        crate::sound::store::RACE_SOUNDS.objects[0].revs.set(0);
        input_update();
        crate::drive::finish_timing::finish_timing();
        let n = RACE_STATE.results.finished.get();
        if n > 1 {
            return Err(BadInput("a finish length past the table's two").into());
        }
        if !c0(
            crate::drive::finish_store::FINISH.time.bits(),
            crate::drive::finish_store::FINISH.finish_lengths[n as usize].bits(),
        ) {
            break;
        }
        let pressed = |key: usize| crate::input::store::DEVICES.keys_pressed.get(key) != 0;
        if pressed(1) || pressed(0x1C) {
            break;
        }
        CAR_BLOCKS.cars[0].camera_view.set_bits(6);
        crate::race::frame_views::load_first_car()?;
        crate::drive::finish_camera::finish_camera();
        crate::physics::camera_frame::camera_frame()?;
        crate::drive::store_first_car::store_first_car()?;
        race_frame()?;
    }
    Ok(())
}

#[cfg(test)]
mod asm_reference {
    use super::*;
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
            operands: vec![Operand::Buffer {
                reads: &[(0x24BB38, In::F), (0x1C5D98, In::F), (0x30F524, In::I)],
                writes: &[(0x30F524, Out::I)],
            }],
            ulps: 0,
            reference: |a| unsafe { ref_lift(a[0]) },
            plain: |a| lift_car(a[0]),
        }];
        sparse(&blocks, 4000, 204);
    }
}
