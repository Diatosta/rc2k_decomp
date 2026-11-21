//! The work car's controls for this step: the keys (`input_words`) into the car's controls and
//! the newly pressed keys; for a player's car (or any car in a network game) bit 3 flips
//! `lights_on` and bit 5 changes the view (`drive::next_camera_view`, not while a replay plays); for a computer's car
//! (`in_credits` set, or `run_kind` and `driver_kind` 2) `drive::drive_computer_car` supplies
//! them. Refuses what `drive_computer_car` refuses (before anything).

use crate::drive::drive_computer_car::drive_computer_car;
use crate::drive::next_camera_view::next_view_at;
use crate::drive::start_blend::start_blend;
use crate::errors::{BadInput, Refused};
use crate::math::x87;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use rc2k_core::drive::controls::{FLIP_BIT, VIEW_BIT, computer_drives, flipped, read_keys};
use rc2k_core::drive::replay_frames::plays_inputs;

/// Does a computer drive this step?
fn computer(shared: &RaceShared, net: &crate::net::owner::Net) -> bool {
    // A test's switch (`tools/remote.py pilot`): the computer at a player's wheel.
    if crate::remote::autodrive()
        && shared.work.get().current_car.get() < shared.view.get().players.get()
    {
        return true;
    }
    // A network race's computer drivers: every machine drives them, the host's messages keep
    // the copies together.
    net.race_slot.is_bot(shared.work.get().current_car.get())
        || computer_drives(
            shared.work.get().in_credits.is_set(),
            || shared.vars.get().run_kind.get(),
            || shared.car().driver_kind.get(),
        )
}

pub fn read_car_controls(
    race: &Race,
    scene: &crate::scene::owner::Scene,
    net: &crate::net::owner::Net,
) -> Result<(), Refused> {
    let race_owner = race;

    let course = &race.course;
    if computer(&race_owner.shared, net)
        && race_owner.shared.vars.get().checkpoint_count.get() == 0
        && race_owner.shared.car().run_ticks.get() >= 0x320
    {
        // `drive_computer_car`'s refusal, before anything here is written.
        let reaches = crate::math::x87::compare_full(
            race_owner.shared.car().crash_matrix.at(5),
            &race_owner.shared.work.get().driver_tilt_limit,
        ) & 0x100
            == 0;
        if reaches {
            return Err(BadInput("no checkpoints for the computer's driving").into());
        }
    }
    let input_keys = race_owner.shared.car().input_keys.get();
    course.lists.borrow_mut().input_keys_mask = input_keys;
    race_owner.shared.car().input_keys.set(0);
    race_owner.shared.car().keys_pressed.set(0);
    // The four input words (recorded or replayed) into the car's inputs.
    let words = [
        &race_owner.shared.car().input_throttle,
        &race_owner.shared.car().input_keys,
        &race_owner.shared.car().input_steering,
        &race_owner.shared.car().input_brake,
    ];
    let input_words = race_owner.shared.run().input_words;
    for (to, from) in words.into_iter().zip(input_words) {
        to.set(from);
    }
    // A held race (`RC2K_HELD_RACE`) takes no key from the players: a key still held from the
    // menus (the test's Return) would reach the car at a tick that depends on the timing.
    if crate::remote::held_race()
        && race_owner.shared.work.get().current_car.get()
            < race_owner.shared.view.get().players.get()
    {
        race_owner.shared.car().input_keys.set(0);
    }
    let keys = read_keys(
        course.lists.borrow().input_keys_mask,
        race_owner.shared.car().input_keys.get(),
    );
    let pressed = keys.pressed;
    race_owner.shared.car().keys_pressed.set(pressed);
    {
        let mut run = race_owner.shared.run_mut();
        let inputs = &mut run.gear_inputs;
        inputs.first = race_owner.shared.car().input_throttle.get();
        inputs.steer = race_owner.shared.car().input_steering.get();
        inputs.second = race_owner.shared.car().input_brake.get();
        inputs.hold = keys.hand;
        inputs.gear_up = keys.up;
        inputs.gear_down = keys.down;
    }
    let network_game = net.race_slot.is_on();
    if network_game
        || race_owner.shared.work.get().current_car.get()
            < race_owner.shared.view.get().players.get()
    {
        if pressed & FLIP_BIT != 0 {
            race_owner
                .shared
                .car()
                .lights_on
                .set(flipped(race_owner.shared.car().lights_on.get()));
        }
        // A replay's recorded change-view presses stay unplayed: the camera is the viewer's.
        let replay_plays = plays_inputs(race_owner.course.number_vars.get().replay_mode.get());
        if pressed & VIEW_BIT != 0 && !replay_plays {
            next_view_at(
                race_owner.kind_tables(),
                race_owner.shared.car(),
                &race_owner.shared.view.get().players,
                &race_owner.shared.car().step_clock,
            )?;
        }
    }
    if !computer(&race_owner.shared, net) {
        return Ok(());
    }
    race_owner
        .shared
        .work
        .get()
        .driver_slot
        .set(race_owner.shared.work.get().current_car.get());
    start_blend(race, scene);
    race_owner
        .shared
        .work
        .get()
        .driver_start_blend
        .set(x87::loaded(&race_owner.shared.car().start_blend));
    // A test's switch (`tools/remote.py pilot`): a player's car has no computer's skill
    // (its blend near 0 holds the wanted speed down), so it drives at the full blend.
    if crate::remote::autodrive()
        && race_owner.shared.work.get().current_car.get()
            < race_owner.shared.view.get().players.get()
    {
        race_owner.shared.work.get().driver_start_blend.set(1.0f32);
    }
    drive_computer_car(race, scene)?;
    {
        let w = race_owner.shared.work.get();
        let mut run = race_owner.shared.run_mut();
        let inputs = &mut run.gear_inputs;
        inputs.steer = w.driver_steer.get();
        inputs.first = w.driver_throttle.get();
        inputs.second = w.driver_brake.get();
        inputs.hold = w.driver_flags[0].get();
        inputs.gear_up = w.driver_flags[1].get();
        inputs.gear_down = w.driver_flags[2].get();
    }
    // A test's switch (`tools/remote.py pilot`): a stalled car reset as a player would,
    // with the reset key.
    if crate::remote::autodrive()
        && race_owner.shared.work.get().current_car.get()
            < race_owner.shared.view.get().players.get()
    {
        let keys =
            crate::remote::autodrive_keys(race_owner, race_owner.shared.car().checkpoint.get());
        if keys != 0 {
            // Off the throttle and on the brake while the key is held, as a player would:
            // the reset needs the car slow throughout.
            let mut run = race_owner.shared.run_mut();
            run.gear_inputs.first = 0;
            run.gear_inputs.second = 0x100;
            run.gear_inputs.steer = 0;
        }
        race_owner
            .shared
            .car()
            .input_keys
            .set(race_owner.shared.car().input_keys.get() | keys);
    }
    Ok(())
}
