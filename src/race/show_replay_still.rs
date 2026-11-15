//! `race::show_replay_still`: draws the viewer's still frame. The frame's block (0x2810 bytes) is
//! copied into the replay's staging area and its cars and cameras are loaded
//! (`drive::load_replay_cars`, `drive::replay_cameras`). The cars are stepped up to the shown
//! frame (`race::step_replay_cars`), with one extra step at frame 0. Each car is then loaded,
//! its camera worked out when it runs and its driver kind is neither 0 nor 2
//! (`physics::camera_frame`), and stored again. The race scene is drawn
//! (`race::draw_race_scene`) with `scene_switch` set to 1 and put back after.
//!
//! Called by `race::viewer_frame`. Refuses (before anything) a cars' count of 0 or past 8.

use crate::errors::{BadInput, Refused};
use crate::memory::cells::copy_cells;
use crate::physics::flow::g;
use crate::race::owner::Race;
use rc2k_core::drive::replay_frames::FRAME_BYTES;

/// The frame's block copied to the replay's (`drive::load_replay_cars` reads it).
fn load_frame(race: &Race) {
    let replay = &race.replay;
    copy_cells(
        (&race.viewer_buffers().block, 0),
        (replay.staging(), 0),
        FRAME_BYTES,
    );
}

pub fn show_replay_still(race: &Race, o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let scene = o.scene;
    use crate::drive::*;
    if !(1..=8).contains(&race.shared.view.get().race_cars.get()) {
        return Err(BadInput("a cars' count of 0 or past 8").into());
    }
    race.physics.split_tables().still.set(1);
    race.frame.frame_work.get().replay_still_on.set(1);
    load_frame(race);
    load_replay_cars::load_replay_cars(race);
    replay_cameras::replay_cameras(race)?;
    for step in 0..g(&race.replay.replay_view().replay_frame) {
        race.replay.replay_frames().step_frame.set(step);
        race.replay
            .replay_frames()
            .step_block
            .set(race.replay.replay_view().replay_block.get());
        crate::race::step_replay_cars::step_replay_cars(o, race)?;
    }
    if race.replay.replay_view().replay_frame.get() == 0 {
        race.replay.replay_frames().step_first.set(1);
        crate::race::step_replay_cars::step_replay_cars(o, race)?;
        race.replay.replay_frames().step_first.set(0);
        load_frame(race);
        load_replay_cars::load_replay_cars(race);
    }
    race.shared.work.get().current_car.set(0);
    loop {
        load_car::load_car(&race.shared)?;
        if !matches!(race.shared.car().driver_kind.get(), 0 | 2) {
            race.replay.replay_frames().step_first.set(1);
            crate::physics::camera_frame::camera_frame(
                &crate::physics::step_inputs::StepInputs::of(race).camera(),
                scene,
            )?;
            race.replay.replay_frames().step_first.set(0);
        }
        store_car::store_car(&race.shared)?;
        race.shared
            .work
            .get()
            .current_car
            .set(race.shared.work.get().current_car.get().wrapping_add(1));
        if race.shared.work.get().current_car.get() == race.shared.view.get().race_cars.get() {
            break;
        }
    }
    race.shared.view.get().view_on[0].set(0);
    race.shared.view.get().view_on[1].set(0);
    race.frame.frame_work.get().drawing.set(1);
    let kept = race.shared.vars.get().scene_switch.get();
    race.shared.vars.get().scene_switch.set(1);
    crate::race::draw_race_scene::draw_race_scene(race, o)?;
    race.shared.vars.get().scene_switch.set(kept);
    race.frame.frame_work.get().drawing.set(0);
    race.frame.frame_work.get().replay_still_on.set(0);
    race.physics.split_tables().still.set(0);
    Ok(())
}
