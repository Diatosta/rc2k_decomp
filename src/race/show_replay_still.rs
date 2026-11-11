//! `race::show_replay_still` (0x46062E): the viewer's still frame, the block read and the frame
//! shown (`replay_frame`) drawn: `still` and `replay_still_on` 1 (a still); the block
//! (`block`, 0x2810 bytes) into the replay's frame (`unk_7172EC`), its state loaded
//! (`drive::load_replay_cars`) and its cameras (`drive::replay_cameras`); the cars stepped to the
//! frame (`race::step_replay_cars` per frame, `step_frame` the frame, `step_block` the block);
//! at frame 0 one step with `dword_6EF7CC` 1 and the block's state loaded again; then each car
//! (`g_currentCar`) loaded (`drive::load_car`), its camera worked out when it runs and is not 2
//! (`physics::camera_frame`, `dword_6EF7CC` 1), stored (`drive::store_car`); `view_on`,
//! `dword_63DB78` 0, `drawing` 1, the race's scene drawn (0x454229: `race::draw_race_scene`)
//! with `scene_switch` 1 (put back after), the three switches 0.
//!
//! Caller: `race::viewer_frame`. Refuses (before anything) a cars' count of 0 or past 8.

use crate::errors::{BadInput, Refused};
use crate::physics::flow::g;
use crate::pointers::copy_cells;
use crate::race::blocks_store::REPLAY_STATE_OFFSET;
use crate::race::owner::Race;
use crate::race::owner_replay::Replay;
use crate::race::viewer_buffers_store::VIEWER_BUFFERS;

/// The frame's block copied to the replay's (`drive::load_replay_cars` reads it).
fn load_frame(replay: &Replay) {
    // unk_7172EC: car block 8's start (the copy runs on past the block, over the next
    // blocks' tail and the replay's tape).
    copy_cells(
        (&VIEWER_BUFFERS.block, 0),
        (&replay.cars_and_replay, REPLAY_STATE_OFFSET),
        4 * 0xA04,
    );
}

pub fn show_replay_still(
    race: &Race,
    input: &crate::input::owner::Input,
    system: &crate::system::owner::System,
) -> Result<(), Refused> {
    use crate::drive::*;
    if !(1..=8).contains(&race.shared.view.race_cars.get()) {
        return Err(BadInput("a cars' count of 0 or past 8").into());
    }
    race.physics.split_tables.still.set(1);
    race.frame.frame_work.replay_still_on.set(1);
    load_frame(&race.replay);
    load_replay_cars::load_replay_cars(race);
    replay_cameras::replay_cameras(race)?;
    for step in 0..g(&race.replay.replay_view.replay_frame) {
        race.replay.replay_frames.step_frame.set(step);
        race.replay
            .replay_frames
            .step_block
            .set(race.replay.replay_view.replay_block.get());
        crate::race::step_replay_cars::step_replay_cars(race, input)?;
    }
    if race.replay.replay_view.replay_frame.get() == 0 {
        race.replay.replay_frames.step_first.set(1);
        crate::race::step_replay_cars::step_replay_cars(race, input)?;
        race.replay.replay_frames.step_first.set(0);
        load_frame(&race.replay);
        load_replay_cars::load_replay_cars(race);
    }
    race.shared.work.current_car.set(0);
    loop {
        load_car::load_car(&race.shared)?;
        if !matches!(race.shared.car.driver_kind.get(), 0 | 2) {
            race.replay.replay_frames.step_first.set(1);
            crate::physics::camera_frame::camera_frame(race, crate::scene::owner::scene())?;
            race.replay.replay_frames.step_first.set(0);
        }
        store_car::store_car(&race.shared)?;
        race.shared
            .work
            .current_car
            .set(race.shared.work.current_car.get().wrapping_add(1));
        if race.shared.work.current_car.get() == race.shared.view.race_cars.get() {
            break;
        }
    }
    race.shared.view.view_on[0].set(0);
    race.shared.view.view_on[1].set(0);
    race.frame.frame_work.drawing.set(1);
    let kept = race.shared.vars.scene_switch.get();
    race.shared.vars.scene_switch.set(1);
    // The scene (it reads no register).
    crate::race::draw_race_scene::draw_race_scene(race, input, system)?;
    race.shared.vars.scene_switch.set(kept);
    race.frame.frame_work.drawing.set(0);
    race.frame.frame_work.replay_still_on.set(0);
    race.physics.split_tables.still.set(0);
    Ok(())
}
