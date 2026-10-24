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
//! Its calls through `part` (`CALLEES`); in the game their Rust versions (`sub_454229`,
//! six bytes calling `draw_race_scene`, is not hooked: its Rust version is called). Shadow
//! mode runs the original alone (it draws). Self-test: a copy whose calls go to stand-ins,
//! against the Rust version: frames 0, 1 and 5, cars running or not, car kinds 0 .. 2.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::{BadInput, Refused};
use crate::physics::flow::{g, p};
use crate::race::step_replay_cars::{CARS, CURRENT};
use crate::race::viewer::{BLOCK, BLOCK_DATA, FRAME, move_dwords};

const STILL: usize = 0x601024; // dword_601024
const STILL_2: usize = 0x6416BC; // dword_6416BC
const DRAWING: usize = 0x6416B8; // dword_6416B8
const REPLAY_FRAME: usize = 0x7172EC; // unk_7172EC
const STEP_FRAME: usize = 0x6EF7E4; // dword_6EF7E4
const STEP_BLOCK: usize = 0x6EF7E0; // dword_6EF7E0
const STEP_FIRST: usize = 0x6EF7CC; // dword_6EF7CC
const KIND: usize = 0x71A234; // unk_71A234 (0 parked, 2 not followed)
const SWITCHES: [usize; 2] = [0x63DB74, 0x63DB78];
const SCENE_SWITCH: usize = 0x6235E8; // dword_6235E8

/// The frame's block copied to the replay's (`drive::load_replay_cars` reads it).
fn load_frame() {
    move_dwords(REPLAY_FRAME, BLOCK_DATA, 0xA04);
}

/// The Rust version (the original leaves registers nothing reads).
pub fn show_replay_still() -> Result<(), Refused> {
    use crate::drive::*;
    if !(1..=8).contains(&g(CARS)) {
        return Err(BadInput("a cars' count of 0 or past 8").into());
    }
    unsafe {
        p(STILL, 1);
        p(STILL_2, 1);
        load_frame();
        load_replay_cars::load_replay_cars();
        replay_cameras::replay_cameras()?;
        for step in 0..g(FRAME) {
            p(STEP_FRAME, step);
            p(STEP_BLOCK, g(BLOCK));
            crate::race::step_replay_cars::step_replay_cars()?;
        }
        if g(FRAME) == 0 {
            p(STEP_FIRST, 1);
            crate::race::step_replay_cars::step_replay_cars()?;
            p(STEP_FIRST, 0);
            load_frame();
            load_replay_cars::load_replay_cars();
        }
        p(CURRENT, 0);
        loop {
            load_car::load_car()?;
            if !matches!(g(KIND), 0 | 2) {
                p(STEP_FIRST, 1);
                crate::physics::camera_frame::camera_frame()?;
                p(STEP_FIRST, 0);
            }
            store_car::store_car()?;
            p(CURRENT, g(CURRENT).wrapping_add(1));
            if g(CURRENT) == g(CARS) {
                break;
            }
        }
        for at in SWITCHES {
            p(at, 0);
        }
        p(DRAWING, 1);
        let kept = g(SCENE_SWITCH);
        p(SCENE_SWITCH, 1);
        // The scene (it reads no register).
        crate::race::draw_race_scene::draw_race_scene()?;
        p(SCENE_SWITCH, kept);
        p(DRAWING, 0);
        p(STILL_2, 0);
        p(STILL, 0);
    }
    Ok(())
}
