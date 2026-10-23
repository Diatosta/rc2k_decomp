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
use crate::framework::{EAX, ESI, Regs};
use crate::physics::flow::{g, p};
use crate::race::viewer::{Part, regs_entry};
use crate::screens::harness::Left;

/// The calls by place, as IDA shows them.
pub const CALLEES: [usize; 13] = [
    0, 0x4560EA, // 1 drive::choose_shown_cars
    0x43A27E, // 2 drive::car_input_words
    0x460162, // 3 drive::play_inputs
    0x460C40, // 4 drive::load_car
    0x43A35A, // 5 drive::read_car_controls
    0x439BF6, // 6 physics::gear_input_rules
    0x439DB1, // 7 drive::steer_inputs
    0x43A004, // 8 drive::pedal_inputs
    0x43A0D3, // 9 drive::reset_request
    0x43B635, // 10 physics::car_physics_step
    0x460C5D, // 11 drive::store_car
    0x436758, // 12 drive::race_mode_cars
];
pub const CURRENT: usize = 0x71B3F8; // g_currentCar
pub const CARS: usize = 0x63DB90; // g_raceCars
const BLOCKS_OF_CARS: usize = 0x71B3B8; // g_carBlocks
const WORK_BLOCK: usize = 0x71B404; // dword_71B404
pub const RUNNING: usize = 0x71A234; // unk_71A234: the work car runs
const CLOCKS: [usize; 2] = [0x719FE0, 0x719FEC];

/// The Rust version: the registers it leaves.
pub fn step_replay_cars(entry: &Regs, part: Part) -> Result<Regs, Refused> {
    // Converted: `rc2k_core::race::viewer_flow::{check_replay_cars, car_block, car_runs,
    // RUN_STEPS, next_car}`.
    use rc2k_core::race::viewer_flow::{
        RUN_STEPS, RunStep, car_block, car_runs, check_replay_cars, next_car,
    };
    check_replay_cars(g(CARS))?;
    let mut r = *entry;
    part(1, &mut r)?;
    p(CURRENT, 0);
    loop {
        let mut c = r;
        part(2, &mut c)?;
        part(3, &mut c)?;
        c.eax = car_block(g(CURRENT));
        c.esi = g(BLOCKS_OF_CARS + 4 * c.eax as usize);
        p(WORK_BLOCK, c.esi);
        part(4, &mut c)?;
        if car_runs(g(RUNNING)) {
            for step in RUN_STEPS {
                let place = match step {
                    RunStep::Controls => 5,
                    RunStep::GearRules => 6,
                    RunStep::Steering => 7,
                    RunStep::Pedals => 8,
                    RunStep::ResetKey => 9,
                    RunStep::Physics => 10,
                };
                part(place, &mut c)?;
            }
            for at in CLOCKS {
                p(at, g(at).wrapping_add(1));
            }
        }
        part(11, &mut c)?;
        let (next, done) = next_car(g(CURRENT), g(CARS));
        p(CURRENT, next);
        r.eax = next;
        if done {
            break;
        }
    }
    part(12, &mut r)?;
    Ok(r)
}

/// In the game: the Rust versions, as `drive::replay_step` calls them.
pub fn game_part(place: u32, r: &mut Regs) -> Result<(), Refused> {
    use crate::drive::*;
    unsafe {
        match place {
            1 => choose_shown_cars::choose_shown_cars()?.apply(r),
            2 => car_input_words::car_input_words(r)?.apply(r),
            3 => play_inputs::play_inputs(r).apply(r),
            4 => load_car::load_car()?.apply(r),
            5 => {
                let mut c = *r;
                read_car_controls::read_car_controls(&mut c)?;
                Left::nothing().with(EAX, c.eax).with(ESI, c.esi).apply(r);
            }
            6 => {
                let mut c = *r;
                crate::physics::gear_input_rules::gear_input_rules(&mut c);
                r.eax = c.eax;
            }
            7 => steer_inputs::steer_inputs(r).apply(r),
            8 => pedal_inputs::pedal_inputs().apply(r),
            9 => *r = reset_request::reset_request(r, &mut crate::physics::reset_parts::game_part)?,
            10 => crate::physics::car_physics_step::car_physics_step(r)?.apply(r),
            11 => store_car::store_car()?.apply(r),
            _ => {
                *r = race_mode_cars::race_mode_cars(r, &mut |_, r| {
                    *r =
                        start_cars_behind::start_cars_behind(r, &mut start_cars_behind::game_part)?;
                    Ok(())
                })?
            }
        }
    }
    Ok(())
}

/// The Rust version where the typed tables run it (no hook; a refusal is the
/// caller's).
pub fn entry(r: &mut Regs) -> Result<(), crate::errors::Refused> {
    regs_entry(r, &|r| step_replay_cars(r, &mut game_part))
}
