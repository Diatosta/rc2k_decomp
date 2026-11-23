//! `race::show_replay_still`: draws the viewer's still frame. The frame's block (0x2810 bytes) is
//! copied into the replay's staging area and its cars and cameras are loaded
//! (`drive::load_replay_cars`, `drive::replay_cameras`). The cars are stepped up to the shown
//! frame (`race::step_replay_cars`), with one extra step at frame 0. Each car is then loaded,
//! its camera worked out when it runs and its driver kind is neither 0 nor 2
//! (`physics::camera_frame`), and stored again. The race scene is drawn
//! (`race::draw_race_scene`) with `scene_switch` set to 1 and put back after.
//!
//! Refuses (before anything) a cars' count of 0 or past 8.

use crate::errors::{BadInput, Refused};
use crate::physics::flow::g;
use crate::race::owner::Race;

/// The frame's block copied to the replay's (`drive::load_replay_cars` reads it).
fn load_frame(race: &Race) {
    crate::race::replay_staging_store::load_from_block(race);
}

pub fn show_replay_still(race: &Race, o: crate::owners::Owners<'static>) -> Result<(), Refused> {
    let scene = o.scene;
    use crate::drive::*;
    if !(1..=8).contains(&race.shared.view.get().race_cars.get()) {
        return Err(BadInput("a cars' count of 0 or past 8").into());
    }
    race.physics.split_tables_mut().still = 1;
    race.frame.frame_work.borrow_mut().replay_still_on = 1;
    load_frame(race);
    load_replay_cars::load_replay_cars(race);
    replay_cameras::replay_cameras(race)?;
    for step in 0..g(&race.replay.replay_view().replay_frame) {
        race.replay.replay_frames_mut().step_frame = step;
        race.replay.replay_frames_mut().step_block = race.replay.replay_view().replay_block.get();
        crate::race::step_replay_cars::step_replay_cars(o, race)?;
    }
    if race.replay.replay_view().replay_frame.get() == 0 {
        race.replay.replay_frames_mut().step_first = 1;
        crate::race::step_replay_cars::step_replay_cars(o, race)?;
        race.replay.replay_frames_mut().step_first = 0;
        load_frame(race);
        load_replay_cars::load_replay_cars(race);
    }
    race.shared.work_mut().current_car = 0;
    loop {
        load_car::load_car(&race.shared)?;
        if !matches!(race.shared.car().driver_kind, 0 | 2) {
            race.replay.replay_frames_mut().step_first = 1;
            crate::physics::camera_frame::camera_frame(
                &crate::physics::step_inputs::StepInputs::of(race).camera(),
                scene,
                &mut race.shared.car_mut(),
            )?;
            race.replay.replay_frames_mut().step_first = 0;
        }
        store_car::store_car(&race.shared)?;
        let next = race.shared.work().current_car.wrapping_add(1);
        race.shared.work_mut().current_car = next;
        if race.shared.work().current_car == race.shared.view.get().race_cars.get() {
            break;
        }
    }
    race.shared.view.get().view_on[0].set(0);
    race.shared.view.get().view_on[1].set(0);
    race.frame.frame_work.borrow_mut().drawing = 1;
    let kept = race.shared.vars.get().scene_switch.get();
    race.shared.vars.get().scene_switch.set(1);
    crate::race::draw_race_scene::draw_race_scene(race, o)?;
    race.shared.vars.get().scene_switch.set(kept);
    race.frame.frame_work.borrow_mut().drawing = 0;
    race.frame.frame_work.borrow_mut().replay_still_on = 0;
    race.physics.split_tables_mut().still = 0;
    Ok(())
}
