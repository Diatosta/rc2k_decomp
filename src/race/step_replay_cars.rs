//! `race::step_replay_cars` (0x460835): one step of the replay's cars, for the viewer's
//! still frame: the cars shown (`drive::choose_shown_cars`); for each car
//! (`g_currentCar` from 0, in `pusha` / `popa`) its input words (`drive::car_input_words`,
//! `drive::play_inputs`), its block into `dword_71B404`, the work car loaded
//! (`drive::load_car`); running (`unk_71A234`): controls, gear rules, steering, pedals,
//! the reset key, `physics::car_physics_step` and the work car's clocks (`unk_719FE0`,
//! `unk_719FEC`) one more; `drive::store_car`. Then `drive::race_mode_cars`.
//!
//! In:  nothing
//! Out: the registers as `race_mode_cars` leaves them; no flags read
//!
//! Caller: `race::show_replay_still` (two). Unlike `drive::replay_step` the clocks count
//! only for a running car, and nothing is recorded or sent. Refuses (before anything) a
//! cars' count of 0 or past 8 (0 goes round 2^32 times). Its calls through `part`
//! (`CALLEES`); in the game their Rust versions (as `drive::replay_step`'s). Shadow mode
//! runs the original alone (as `car_physics_step`). Self-test: a copy whose calls go to
//! stand-ins (the running switch by car), against the Rust version: one, three and eight
//! cars.
//!
//! Hook removed 2026-10-23 (conversion phase 4): Rust callers call this directly; in the
//! hooked DLL the game's own code runs the original. What the header says of shadow mode
//! and the self-test was the hook's.

use crate::errors::Refused;
use crate::physics::flow::{g, p};

pub const CURRENT: usize = 0x71B3F8; // g_currentCar
pub const CARS: usize = 0x63DB90; // g_raceCars
const BLOCKS_OF_CARS: usize = 0x71B3B8; // g_carBlocks
const WORK_BLOCK: usize = 0x71B404; // dword_71B404
pub const RUNNING: usize = 0x71A234; // unk_71A234: the work car runs
const CLOCKS: [usize; 2] = [0x719FE0, 0x719FEC];

/// The Rust version (the original leaves registers nothing reads).
pub fn step_replay_cars() -> Result<(), Refused> {
    use crate::drive::*;
    // Converted: `rc2k_core::race::viewer_flow::{check_replay_cars, car_block, car_runs,
    // RUN_STEPS, next_car}`.
    use rc2k_core::race::viewer_flow::{
        RUN_STEPS, RunStep, car_block, car_runs, check_replay_cars, next_car,
    };
    check_replay_cars(g(CARS))?;
    unsafe {
        choose_shown_cars::choose_shown_cars()?;
        p(CURRENT, 0);
        loop {
            car_input_words::car_input_words()?;
            play_inputs::play_inputs();
            p(
                WORK_BLOCK,
                g(BLOCKS_OF_CARS + 4 * car_block(g(CURRENT)) as usize),
            );
            load_car::load_car()?;
            if car_runs(g(RUNNING)) {
                for step in RUN_STEPS {
                    match step {
                        RunStep::Controls => read_car_controls::read_car_controls()?,
                        RunStep::GearRules => crate::physics::gear_input_rules::gear_input_rules(),
                        RunStep::Steering => steer_inputs::steer_inputs(),
                        RunStep::Pedals => pedal_inputs::pedal_inputs(),
                        RunStep::ResetKey => reset_request::reset_request()?,
                        RunStep::Physics => crate::physics::car_physics_step::car_physics_step()?,
                    }
                }
                for at in CLOCKS {
                    p(at, g(at).wrapping_add(1));
                }
            }
            store_car::store_car()?;
            let (next, done) = next_car(g(CURRENT), g(CARS));
            p(CURRENT, next);
            if done {
                break;
            }
        }
        race_mode_cars::race_mode_cars()
    }
}
