//! `drive::read_car_controls` (0x43A35A): the work car's controls for this step:
//! - the keys (`dword_6138C4` .. `D0`) into the car's (`dword_71A43C` .. `48`), the
//!   held-before into `dword_5C694C` and the newly pressed (`dword_71A44C`, bits not held
//!   before); steering, throttle and brake into 0x613890 .. 0x613898, and three bits as
//!   0x61389C (bit 2 times 0x60), 0x6138A0, 0x6138A4;
//! - for a player's car (or in a network game, `dword_52A910`, whatever the car): bit 3 newly pressed flips
//!   `dword_71A05C`, bit 5 changes the view (`drive::next_camera_view` on `g_car`);
//! - for a computer's car (`dword_71C3E0` set, or `dword_62389C` and `dword_71A234` 2):
//!   `drive::start_blend`, `flt_71BFA4` = `flt_71A658`, `drive::drive_computer_car`, and
//!   its six controls (0x71BF88 ..) into 0x613890 .. 0x6138A4.
//!
//! The bytes after its `retn` (0x43A4E5 ..) are another entry that nothing calls.
//!
//! Callers: race init's loop (`drive::race_cars_step`), `race::step_replay_cars`,
//! `drive::replay_step`, `drive::simple_car_step`. Refuses what `drive_computer_car` refuses
//! (before anything).

use crate::drive::drive_computer_car::drive_computer_car;
use crate::drive::next_camera_view::next_view_at;
use crate::drive::start_blend::start_blend;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::net::owner::net;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use rc2k_core::drive::controls::{FLIP_BIT, VIEW_BIT, computer_drives, flipped, read_keys};

/// Does a computer drive this step?
fn computer(shared: &RaceShared) -> bool {
    // A test's switch (`tools/remote.py pilot`): the computer at a player's wheel.
    #[cfg(feature = "remote")]
    if crate::remote::autodrive() && shared.work.current_car.get() < shared.view.players.get() {
        return true;
    }
    computer_drives(
        shared.work.in_credits.get(),
        || shared.vars.run_kind.get(),
        || shared.car.driver_kind.get(),
    )
}

/// `r` as the original leaves it.
// Converted: `rc2k_core::drive::controls::{read_keys, computer_drives, flipped}`.
pub fn read_car_controls(race: &Race) -> Result<(), Refused> {
    let race_owner = race;

    let course = &race.course;
    if computer(&race_owner.shared)
        && race_owner.shared.vars.checkpoint_count.get() == 0
        && race_owner.shared.car.run_ticks.get() >= 0x320
    {
        // `drive_computer_car`'s refusal, before anything here is written.
        let reaches = crate::math::x87_run::compare_full(
            race_owner.shared.car.crash_matrix[5].bits(),
            race_owner.shared.work.driver_tilt_limit.bits(),
        ) & 0x100
            == 0;
        if reaches {
            return Err(BadInput("no checkpoints for the computer's driving").into());
        }
    }
    course
        .lists
        .input_keys_mask
        .set(race_owner.shared.car.input_keys.get());
    race_owner.shared.car.input_keys.set(0);
    race_owner.shared.car.keys_pressed.set(0);
    // The four input words (recorded or replayed) into the car's inputs.
    let words = [
        &race_owner.shared.car.input_throttle,
        &race_owner.shared.car.input_keys,
        &race_owner.shared.car.input_steering,
        &race_owner.shared.car.input_brake,
    ];
    for (to, from) in words
        .into_iter()
        .zip(&race_owner.shared.physics_tables.input_words)
    {
        to.set(from.get());
    }
    // A held race (`RC2K_HELD_RACE`) takes no key from the players: a key still held from the
    // menus (the test's Return) would reach the car at a tick that depends on the timing.
    #[cfg(feature = "remote")]
    if crate::remote::held_race()
        && race_owner.shared.work.current_car.get() < race_owner.shared.view.players.get()
    {
        race_owner.shared.car.input_keys.set(0);
    }
    let keys = read_keys(
        course.lists.input_keys_mask.get(),
        race_owner.shared.car.input_keys.get(),
    );
    let pressed = keys.pressed;
    race_owner.shared.car.keys_pressed.set(pressed);
    race_owner
        .shared
        .physics_tables
        .input_throttle
        .set(race_owner.shared.car.input_throttle.get());
    race_owner
        .shared
        .physics_tables
        .input_steer
        .set(race_owner.shared.car.input_steering.get());
    race_owner
        .shared
        .physics_tables
        .input_brake
        .set(race_owner.shared.car.input_brake.get());
    race_owner
        .shared
        .physics_tables
        .input_handbrake
        .set(keys.hand);
    race_owner.shared.physics_tables.input_gear_up.set(keys.up);
    race_owner
        .shared
        .physics_tables
        .input_gear_down
        .set(keys.down);
    let replay = net().vars.session.get() != 0;
    if replay || race_owner.shared.work.current_car.get() < race_owner.shared.view.players.get() {
        if pressed & FLIP_BIT != 0 {
            race_owner
                .shared
                .car
                .lights_on
                .set(flipped(race_owner.shared.car.lights_on.get()));
        }
        if pressed & VIEW_BIT != 0 {
            next_view_at(
                &race_owner.shared.car,
                &race_owner.shared.view.players,
                &race_owner.shared.car.step_clock,
            )?;
        }
    }
    if !computer(&race_owner.shared) {
        return Ok(());
    }
    race_owner
        .shared
        .work
        .driver_slot
        .set(race_owner.shared.work.current_car.get());
    start_blend(race);
    race_owner
        .shared
        .work
        .driver_start_blend
        .set_bits(x87::loaded(race_owner.shared.car.start_blend.bits()));
    // A test's switch (`tools/remote.py pilot`): a player's car has no computer's skill
    // (its blend near 0 holds the wanted speed down), so it drives at the full blend.
    #[cfg(feature = "remote")]
    if crate::remote::autodrive()
        && race_owner.shared.work.current_car.get() < race_owner.shared.view.players.get()
    {
        race_owner.shared.work.driver_start_blend.set(1.0f32);
    }
    drive_computer_car(race)?;
    let (ph, w) = (&race_owner.shared.physics_tables, &race_owner.shared.work);
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
        && race_owner.shared.work.current_car.get() < race_owner.shared.view.players.get()
    {
        let keys = crate::remote::autodrive_keys(race_owner.shared.car.checkpoint.get());
        if keys != 0 {
            // Off the throttle and on the brake while the key is held, as a player would:
            // the reset needs the car slow throughout.
            race_owner.shared.physics_tables.input_throttle.set(0);
            race_owner.shared.physics_tables.input_brake.set(0x100);
            race_owner.shared.physics_tables.input_steer.set(0);
        }
        race_owner
            .shared
            .car
            .input_keys
            .set(race_owner.shared.car.input_keys.get() | keys);
    }
    Ok(())
}
