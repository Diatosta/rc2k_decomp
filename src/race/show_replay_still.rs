//! `race::show_replay_still` (0x46062E): the viewer's still frame, the block read and the
//! frame shown (`dword_5C2534`) drawn: `dword_601024` and `dword_6416BC` 1 (a still); the
//! block (`dword_9613A4`, 0x2810 bytes) into the replay's frame (`unk_7172EC`), its state
//! loaded (`drive::load_replay_cars`) and its cameras (`drive::replay_cameras`); the cars
//! stepped to the frame (`race::step_replay_cars` per frame, `dword_6EF7E4` the frame,
//! `dword_6EF7E0` the block); at frame 0 one step with `dword_6EF7CC` 1 and the block's
//! state loaded again; then each car (`g_currentCar`) loaded (`drive::load_car`), its
//! camera worked out when it runs and is not 2 (`physics::camera_frame`, `dword_6EF7CC`
//! 1), stored (`drive::store_car`); `dword_63DB74`, `dword_63DB78` 0, `dword_6416B8` 1,
//! the race's scene drawn (`sub_454229`: `race::draw_race_scene`) with `dword_6235E8` 1
//! (put back after), the three switches 0.
//!
//! In:  nothing
//! Out: the registers as the scene's drawing leaves them; no flags read
//!
//! Caller: `race::viewer_frame`. Refuses (before anything) a cars' count of 0 or past 8.

use crate::drive::replay_frames_store::REPLAY_FRAMES;
use crate::errors::{BadInput, Refused};
use crate::physics::car_store::CAR;
use crate::physics::flow::g;
use crate::pointers::copy_cells;
use crate::race::blocks_store::REPLAY_STATE_OFFSET;
use crate::race::frame_store::FRAME_WORK;
use crate::race::groups_store::CARS_AND_REPLAY;
use crate::race::vars_store::RACE_VARS;
use crate::race::view_store::RACE_VIEW;
use crate::race::viewer_buffers_store::VIEWER_BUFFERS;
use crate::race::viewer_state_store::REPLAY_VIEW;
use crate::race::work_store::RACE_WORK;

/// The frame's block copied to the replay's (`drive::load_replay_cars` reads it).
fn load_frame() {
    // unk_7172EC: car block 8's start (the copy runs on past the block, over the next
    // blocks' tail and the replay's tape).
    copy_cells(
        (&VIEWER_BUFFERS.block, 0),
        (&CARS_AND_REPLAY, REPLAY_STATE_OFFSET),
        4 * 0xA04,
    );
}

/// The Rust version (the original leaves registers nothing reads).
pub fn show_replay_still() -> Result<(), Refused> {
    use crate::drive::*;
    if !(1..=8).contains(&RACE_VIEW.race_cars.get()) {
        return Err(BadInput("a cars' count of 0 or past 8").into());
    }
    crate::physics::split_tables_store::SPLIT_TABLES
        .still
        .set(1);
    FRAME_WORK.still_2.set(1);
    load_frame();
    load_replay_cars::load_replay_cars();
    replay_cameras::replay_cameras()?;
    for step in 0..g(&REPLAY_VIEW.replay_frame) {
        REPLAY_FRAMES.step_frame.set(step);
        REPLAY_FRAMES.step_block.set(REPLAY_VIEW.replay_block.get());
        crate::race::step_replay_cars::step_replay_cars()?;
    }
    if REPLAY_VIEW.replay_frame.get() == 0 {
        REPLAY_FRAMES.step_first.set(1);
        crate::race::step_replay_cars::step_replay_cars()?;
        REPLAY_FRAMES.step_first.set(0);
        load_frame();
        load_replay_cars::load_replay_cars();
    }
    RACE_WORK.current_car.set(0);
    loop {
        load_car::load_car()?;
        if !matches!(CAR.driver_kind.get(), 0 | 2) {
            REPLAY_FRAMES.step_first.set(1);
            crate::physics::camera_frame::camera_frame()?;
            REPLAY_FRAMES.step_first.set(0);
        }
        store_car::store_car()?;
        RACE_WORK
            .current_car
            .set(RACE_WORK.current_car.get().wrapping_add(1));
        if RACE_WORK.current_car.get() == RACE_VIEW.race_cars.get() {
            break;
        }
    }
    RACE_VIEW.view_on[0].set(0);
    RACE_VIEW.view_on[1].set(0);
    FRAME_WORK.drawing.set(1);
    let kept = RACE_VARS.scene_switch.get();
    RACE_VARS.scene_switch.set(1);
    // The scene (it reads no register).
    crate::race::draw_race_scene::draw_race_scene()?;
    RACE_VARS.scene_switch.set(kept);
    FRAME_WORK.drawing.set(0);
    FRAME_WORK.still_2.set(0);
    crate::physics::split_tables_store::SPLIT_TABLES
        .still
        .set(0);
    Ok(())
}
