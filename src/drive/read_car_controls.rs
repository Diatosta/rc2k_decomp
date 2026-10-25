//! `drive::read_car_controls` (0x43A35A): the work car's controls for this step:
//! - the keys (`dword_6138C4` .. `D0`) into the car's (`dword_71A43C` .. `48`), the
//!   held-before into `dword_5C694C` and the newly pressed (`dword_71A44C`, bits not held
//!   before); steering, throttle and brake into 0x613890 .. 0x613898, and three bits as
//!   0x61389C (bit 2 times 0x60), 0x6138A0, 0x6138A4;
//! - for a player's car (or in a replay, `dword_52A910`): bit 3 newly pressed flips
//!   `dword_71A05C`, bit 5 changes the view (`drive::next_camera_view` on `g_car`);
//! - for a computer's car (`dword_71C3E0` set, or `dword_62389C` and `dword_71A234` 2):
//!   `drive::start_blend`, `flt_71BFA4` = `flt_71A658`, `drive::drive_computer_car`, and
//!   its six controls (0x71BF88 ..) into 0x613890 .. 0x6138A4.
//!
//! The bytes after its `retn` (0x43A4E5 ..) are another entry that nothing calls.
//!
//! In:  nothing
//! Out: eax as the path leaves it (the last bit's, `next_camera_view`'s, or
//!      `start_blend`'s count); esi `g_car` when the view changed; the rest kept; no
//!      flags read (callers go on with calls)
//!
//! Callers: race init's loop (`sub_4608BB`), `sub_460835`, `sub_460B2F`, `sub_43A1CA`.
//! Refuses what `drive_computer_car` refuses (before anything).

use crate::drive::drive_computer_car::drive_computer_car;
use crate::drive::next_camera_view::next_camera_view;
use crate::drive::start_blend::start_blend;
use crate::errors::{BadInput, Refused};
use crate::framework::{game_read, game_write, rva};
use crate::math::x87;
use crate::physics::car_store::CAR;
use crate::physics::tables_store::PHYSICS;
use crate::race::vars_store::RACE_VARS;
use crate::race::view_store::RACE_VIEW;
use crate::race::work_store::RACE_WORK;
use rc2k_core::drive::controls::{FLIP_BIT, VIEW_BIT, computer_drives, flipped, read_keys};

fn g(x: u32) -> u32 {
    unsafe { game_read(rva(x as usize)) }
}

fn p(x: u32, v: u32) {
    unsafe { game_write(rva(x as usize), v) }
}

/// Does a computer drive this step?
fn computer() -> bool {
    computer_drives(
        RACE_WORK.in_credits.get(),
        || RACE_VARS.run_kind.get(),
        || CAR.driver_kind.get(),
    )
}

/// The Rust version: `r` as the original leaves it.
// Converted: `rc2k_core::drive::controls::{read_keys, computer_drives, flipped}`.
pub unsafe fn read_car_controls() -> Result<(), Refused> {
    unsafe {
        if computer() && RACE_VARS.checkpoint_count.get() == 0 && CAR.run_ticks.get() >= 0x320 {
            // `drive_computer_car`'s refusal, before anything here is written.
            let reaches = crate::math::x87_run::compare_full(
                CAR.crash_matrix[5].bits(),
                RACE_WORK.driver_tilt_limit.bits(),
            ) & 0x100
                == 0;
            if reaches {
                return Err(BadInput("no checkpoints for the computer's driving").into());
            }
        }
        p(0x5C694C, CAR.input_keys.get());
        CAR.input_keys.set(0);
        CAR.keys_pressed.set(0);
        // The four input words (recorded or replayed) into the car's inputs.
        let words = [
            &CAR.input_throttle,
            &CAR.input_keys,
            &CAR.input_steering,
            &CAR.input_brake,
        ];
        for (to, from) in words.into_iter().zip(&PHYSICS.input_words) {
            to.set(from.get());
        }
        let keys = read_keys(g(0x5C694C), CAR.input_keys.get());
        let pressed = keys.pressed;
        CAR.keys_pressed.set(pressed);
        PHYSICS.input_throttle.set(CAR.input_throttle.get());
        PHYSICS.input_steer.set(CAR.input_steering.get());
        PHYSICS.input_brake.set(CAR.input_brake.get());
        PHYSICS.input_handbrake.set(keys.hand);
        PHYSICS.input_gear_up.set(keys.up);
        PHYSICS.input_gear_down.set(keys.down);
        let replay = crate::net::store::NET.session.get() != 0;
        if replay || RACE_WORK.current_car.get() < RACE_VIEW.players.get() {
            if pressed & FLIP_BIT != 0 {
                CAR.lights_on.set(flipped(CAR.lights_on.get()));
            }
            if pressed & VIEW_BIT != 0 {
                next_camera_view(crate::physics::car_store::car_addr() as u32);
            }
        }
        if !computer() {
            return Ok(());
        }
        RACE_WORK.driver_slot.set(RACE_WORK.current_car.get());
        start_blend();
        RACE_WORK
            .driver_start_blend
            .set_bits(x87::loaded(CAR.start_blend.bits()));
        drive_computer_car()?;
        let (ph, w) = (&PHYSICS, &RACE_WORK);
        for (to, from) in [
            (&ph.input_steer, &w.driver_steer),
            (&ph.input_throttle, &w.driver_throttle),
            (&ph.input_brake, &w.driver_brake),
            (&ph.input_handbrake, &w.driver_flags[0]),
            (&ph.input_gear_up, &w.driver_flags[1]),
            (&ph.input_gear_down, &w.driver_flags[2]),
        ] {
            to.set(from.get());
        }
        Ok(())
    }
}
