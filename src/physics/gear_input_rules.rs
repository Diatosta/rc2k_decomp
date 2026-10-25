//! `physics::gear_input_rules` (0x439BF6): the gearbox's input state (0x613890 ..
//! 0x6138A8) by the car's state (the work copy's `dword_71A244`, `dword_71A010`,
//! `dword_719FEC`, `dword_71A07C`, `dword_71A078`, `dword_71A080`): with `dword_71A244`
//! neither 0 nor -1 all cleared, 0x61389C = 500 and `g_carGear` 0 (nothing else); with
//! `dword_71A010` 2 the two inputs 0x613894 / 0x613898 joined into the first (in neutral,
//! `dword_6138A8` = 1, or -1 when the second is the larger) or swapped (in reverse);
//! with `dword_719FEC` below 800 (the sign of the difference) 0x61389C = 500 and, unless
//! `dword_62389C` is 1, four cleared; each of three switches clears them and the gear.
//! What they are is a reading at best (an automatic box's held inputs?).
//!
//! In:  nothing
//! Out: eax the joined input when they were joined; the rest kept; no flags read
//!
//! Callers: race init's loop (`sub_4608BB`), `sub_460835`, `sub_460B2F` (each calls
//! `sub_43B635` or `sub_439DB1` next).
//!
//! Plain since 2026-10-23 (phase 3): its arguments are what it reads; the registers
//! above are the original's, for reference.

use crate::physics::car_store::CAR;
use crate::physics::tables_store::PHYSICS;
use crate::race::vars_store::RACE_VARS;
use rc2k_core::physics::gearbox::{GearCar, GearInputs};

unsafe fn get(v: &[u32], k: usize) -> u32 {
    unsafe { (v[k] as *const u32).read_unaligned() }
}

unsafe fn put(v: &[u32], k: usize, x: u32) {
    unsafe { (v[k] as *mut u32).write_unaligned(x) }
}

/// The Rust version on the variables at `v`; what the rules change is written back. The
/// eax it leaves (the joined input) is read by none of its callers.
// Converted: `rc2k_core::physics::gearbox::GearInputs::apply_rules`.
pub unsafe fn rules_at(v: &[u32]) {
    unsafe {
        let car = GearCar {
            state: get(v, 0),
            box_mode: get(v, 7),
            timer: get(v, 9),
            race_mode: get(v, 10),
            switches: [get(v, 12), get(v, 13), get(v, 14)],
        };
        let before = GearInputs {
            w90: get(v, 11),
            first: get(v, 3),
            second: get(v, 4),
            hold: get(v, 5),
            wa0: get(v, 2),
            wa4: get(v, 1),
            joined: get(v, 8),
        };
        let gear_before = get(v, 6);
        let (mut inputs, mut gear) = (before, gear_before);
        inputs.apply_rules(&mut gear, &car);
        for (k, now, was) in [
            (11, inputs.w90, before.w90),
            (3, inputs.first, before.first),
            (4, inputs.second, before.second),
            (5, inputs.hold, before.hold),
            (2, inputs.wa0, before.wa0),
            (1, inputs.wa4, before.wa4),
            (8, inputs.joined, before.joined),
            (6, gear, gear_before),
        ] {
            if now != was {
                put(v, k, now);
            }
        }
    }
}

fn game() -> Vec<u32> {
    vec![
        CAR.tyre_change_state.addr() as u32,
        PHYSICS.input_gear_down.addr() as u32,
        PHYSICS.input_gear_up.addr() as u32,
        PHYSICS.input_throttle.addr() as u32,
        PHYSICS.input_brake.addr() as u32,
        PHYSICS.input_handbrake.addr() as u32,
        CAR.gear.addr() as u32,
        CAR.gearbox_kind.addr() as u32,
        PHYSICS.input_gear_wanted.addr() as u32,
        CAR.run_ticks.addr() as u32,
        RACE_VARS.run_kind.addr() as u32,
        PHYSICS.input_steer.addr() as u32,
        CAR.go_stand_timer.addr() as u32,
        CAR.started_stand_timer.addr() as u32,
        CAR.finish_flag.addr() as u32,
    ]
}

/// The Rust version.
pub unsafe fn gear_input_rules() {
    unsafe { rules_at(&game()) }
}
