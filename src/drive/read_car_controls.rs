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
use crate::drive::next_camera_view::next_view_at;
use crate::drive::start_blend::start_blend;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::owner::race;
use rc2k_core::drive::controls::{FLIP_BIT, VIEW_BIT, computer_drives, flipped, read_keys};

/// Does a computer drive this step?
fn computer() -> bool {
    // A test's switch (`tools/remote.py pilot`): the computer at a player's wheel.
    #[cfg(feature = "remote")]
    if crate::remote::autodrive()
        && crate::race::owner::race().shared.work.current_car.get()
            < crate::race::owner::race().shared.view.players.get()
    {
        return true;
    }
    computer_drives(
        crate::race::owner::race().shared.work.in_credits.get(),
        || crate::race::owner::race().shared.vars.run_kind.get(),
        || crate::race::owner::race().shared.car.driver_kind.get(),
    )
}

/// The Rust version: `r` as the original leaves it.
// Converted: `rc2k_core::drive::controls::{read_keys, computer_drives, flipped}`.
pub fn read_car_controls() -> Result<(), Refused> {
    let race = race();
    let course = &race.course;
    if computer()
        && crate::race::owner::race()
            .shared
            .vars
            .checkpoint_count
            .get()
            == 0
        && crate::race::owner::race().shared.car.run_ticks.get() >= 0x320
    {
        // `drive_computer_car`'s refusal, before anything here is written.
        let reaches = crate::math::x87_run::compare_full(
            crate::race::owner::race().shared.car.crash_matrix[5].bits(),
            crate::race::owner::race()
                .shared
                .work
                .driver_tilt_limit
                .bits(),
        ) & 0x100
            == 0;
        if reaches {
            return Err(BadInput("no checkpoints for the computer's driving").into());
        }
    }
    course
        .lists
        .input_keys_mask
        .set(crate::race::owner::race().shared.car.input_keys.get());
    crate::race::owner::race().shared.car.input_keys.set(0);
    crate::race::owner::race().shared.car.keys_pressed.set(0);
    // The four input words (recorded or replayed) into the car's inputs.
    let words = [
        &crate::race::owner::race().shared.car.input_throttle,
        &crate::race::owner::race().shared.car.input_keys,
        &crate::race::owner::race().shared.car.input_steering,
        &crate::race::owner::race().shared.car.input_brake,
    ];
    for (to, from) in words
        .into_iter()
        .zip(&crate::race::owner::race().shared.physics_tables.input_words)
    {
        to.set(from.get());
    }
    let keys = read_keys(
        course.lists.input_keys_mask.get(),
        crate::race::owner::race().shared.car.input_keys.get(),
    );
    let pressed = keys.pressed;
    crate::race::owner::race()
        .shared
        .car
        .keys_pressed
        .set(pressed);
    crate::race::owner::race()
        .shared
        .physics_tables
        .input_throttle
        .set(crate::race::owner::race().shared.car.input_throttle.get());
    crate::race::owner::race()
        .shared
        .physics_tables
        .input_steer
        .set(crate::race::owner::race().shared.car.input_steering.get());
    crate::race::owner::race()
        .shared
        .physics_tables
        .input_brake
        .set(crate::race::owner::race().shared.car.input_brake.get());
    crate::race::owner::race()
        .shared
        .physics_tables
        .input_handbrake
        .set(keys.hand);
    crate::race::owner::race()
        .shared
        .physics_tables
        .input_gear_up
        .set(keys.up);
    crate::race::owner::race()
        .shared
        .physics_tables
        .input_gear_down
        .set(keys.down);
    let replay = crate::net::owner::net().vars.session.get() != 0;
    if replay
        || crate::race::owner::race().shared.work.current_car.get()
            < crate::race::owner::race().shared.view.players.get()
    {
        if pressed & FLIP_BIT != 0 {
            crate::race::owner::race().shared.car.lights_on.set(flipped(
                crate::race::owner::race().shared.car.lights_on.get(),
            ));
        }
        if pressed & VIEW_BIT != 0 {
            next_view_at(
                &crate::race::owner::race().shared.car,
                &crate::race::owner::race().shared.view.players,
                &crate::race::owner::race().shared.car.step_clock,
            )?;
        }
    }
    if !computer() {
        return Ok(());
    }
    crate::race::owner::race()
        .shared
        .work
        .driver_slot
        .set(crate::race::owner::race().shared.work.current_car.get());
    start_blend(&race.course);
    crate::race::owner::race()
        .shared
        .work
        .driver_start_blend
        .set_bits(x87::loaded(
            crate::race::owner::race().shared.car.start_blend.bits(),
        ));
    // A test's switch (`tools/remote.py pilot`): a player's car has no computer's skill
    // (its blend near 0 holds the wanted speed down), so it drives at the full blend.
    #[cfg(feature = "remote")]
    if crate::remote::autodrive()
        && crate::race::owner::race().shared.work.current_car.get()
            < crate::race::owner::race().shared.view.players.get()
    {
        crate::race::owner::race()
            .shared
            .work
            .driver_start_blend
            .set_bits(1.0f32.to_bits());
    }
    drive_computer_car(race)?;
    let (ph, w) = (
        &crate::race::owner::race().shared.physics_tables,
        &crate::race::owner::race().shared.work,
    );
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
    // A test's switch (`tools/remote.py pilot`): a stalled car reset as a player would,
    // with the reset key.
    #[cfg(feature = "remote")]
    if crate::remote::autodrive()
        && crate::race::owner::race().shared.work.current_car.get()
            < crate::race::owner::race().shared.view.players.get()
    {
        let keys =
            crate::remote::autodrive_keys(crate::race::owner::race().shared.car.checkpoint.get());
        if keys != 0 {
            // Off the throttle and on the brake while the key is held, as a player would:
            // the reset needs the car slow throughout.
            crate::race::owner::race()
                .shared
                .physics_tables
                .input_throttle
                .set(0);
            crate::race::owner::race()
                .shared
                .physics_tables
                .input_brake
                .set(0x100);
            crate::race::owner::race()
                .shared
                .physics_tables
                .input_steer
                .set(0);
        }
        crate::race::owner::race()
            .shared
            .car
            .input_keys
            .set(crate::race::owner::race().shared.car.input_keys.get() | keys);
    }
    Ok(())
}
