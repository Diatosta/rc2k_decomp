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
use crate::memory::plain_floats::FloatRow;
use crate::race::owner::Race;
use crate::race::owner_shared::RaceShared;
use rc2k_core::drive::controls::{FLIP_BIT, VIEW_BIT, computer_drives, flipped, read_keys};
use rc2k_core::drive::replay_frames::plays_inputs;

/// Does a computer drive this step?
fn computer(shared: &RaceShared, net: &crate::net::owner::Net) -> bool {
    // A test's switch (`tools/remote.py pilot`): the computer at a player's wheel.
    if crate::remote::autodrive() && shared.work().current_car < shared.view.get().players.get() {
        return true;
    }
    // A network race's computer drivers: every machine drives them, the host's messages keep
    // the copies together.
    net.race_slot.is_bot(shared.work().current_car)
        || computer_drives(
            shared.work().in_credits != 0,
            || shared.vars.get().run_kind.get(),
            || shared.car().driver_kind,
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
        && race_owner.shared.car().run_ticks >= 0x320
    {
        // `drive_computer_car`'s refusal, before anything here is written.
        let reaches = crate::math::x87::compare_full(
            race_owner.shared.car().crash_matrix.at(5),
            race_owner.shared.work().driver_tilt_limit,
        ) & 0x100
            == 0;
        if reaches {
            return Err(BadInput("no checkpoints for the computer's driving").into());
        }
    }
    let input_keys = race_owner.shared.car().input_keys;
    course.lists.borrow_mut().input_keys_mask = input_keys;
    race_owner.shared.car_mut().input_keys = 0;
    race_owner.shared.car_mut().keys_pressed = 0;
    // The four input words (recorded or replayed) into the car's inputs.
    let input_words = race_owner.shared.run().input_words;
    {
        let mut car = race_owner.shared.car_mut();
        let car = &mut *car;
        let words = [
            &mut car.input_throttle,
            &mut car.input_keys,
            &mut car.input_steering,
            &mut car.input_brake,
        ];
        for (to, from) in words.into_iter().zip(input_words) {
            *to = from;
        }
    }
    // A held race (`RC2K_HELD_RACE`) takes no key from the players: a key still held from the
    // menus (the test's Return) would reach the car at a tick that depends on the timing.
    if crate::remote::held_race()
        && race_owner.shared.work().current_car < race_owner.shared.view.get().players.get()
    {
        race_owner.shared.car_mut().input_keys = 0;
    }
    let keys = read_keys(
        course.lists.borrow().input_keys_mask,
        race_owner.shared.car().input_keys,
    );
    let pressed = keys.pressed;
    race_owner.shared.car_mut().keys_pressed = pressed;
    {
        let mut run = race_owner.shared.run_mut();
        let inputs = &mut run.gear_inputs;
        inputs.first = race_owner.shared.car().input_throttle;
        inputs.steer = race_owner.shared.car().input_steering;
        inputs.second = race_owner.shared.car().input_brake;
        inputs.hold = keys.hand;
        inputs.gear_up = keys.up;
        inputs.gear_down = keys.down;
    }
    let network_game = net.race_slot.is_on();
    if network_game
        || race_owner.shared.work().current_car < race_owner.shared.view.get().players.get()
    {
        if pressed & FLIP_BIT != 0 {
            let value = flipped(race_owner.shared.car().lights_on);
            race_owner.shared.car_mut().lights_on = value;
        }
        // A replay's recorded change-view presses stay unplayed: the camera is the viewer's.
        let replay_plays = plays_inputs(race_owner.course.replay_mode());
        if pressed & VIEW_BIT != 0 && !replay_plays {
            let step = race_owner.shared.car().step_clock;
            next_view_at(
                race_owner.kind_tables(),
                &mut race_owner.shared.car_mut(),
                &race_owner.shared.view.get().players,
                step,
            )?;
        }
    }
    if !computer(&race_owner.shared, net) {
        return Ok(());
    }
    let current_car = race_owner.shared.work().current_car;
    race_owner.shared.work_mut().driver_slot = current_car;
    start_blend(race, scene);
    let blend = x87::loaded(race_owner.shared.car().start_blend);
    race_owner.shared.work_mut().driver_start_blend = blend;
    // A test's switch (`tools/remote.py pilot`): a player's car has no computer's skill
    // (its blend near 0 holds the wanted speed down), so it drives at the full blend.
    if crate::remote::autodrive()
        && race_owner.shared.work().current_car < race_owner.shared.view.get().players.get()
    {
        race_owner.shared.work_mut().driver_start_blend = 1.0f32;
    }
    drive_computer_car(race, scene)?;
    {
        let w = race_owner.shared.work();
        let mut run = race_owner.shared.run_mut();
        let inputs = &mut run.gear_inputs;
        inputs.steer = w.driver_steer;
        inputs.first = w.driver_throttle;
        inputs.second = w.driver_brake;
        inputs.hold = w.driver_flags[0];
        inputs.gear_up = w.driver_flags[1];
        inputs.gear_down = w.driver_flags[2];
    }
    // A test's switch (`tools/remote.py pilot`): a stalled car reset as a player would,
    // with the reset key.
    if crate::remote::autodrive()
        && race_owner.shared.work().current_car < race_owner.shared.view.get().players.get()
    {
        let keys = crate::remote::autodrive_keys(race_owner, race_owner.shared.car().checkpoint);
        if keys != 0 {
            // Off the throttle and on the brake while the key is held, as a player would:
            // the reset needs the car slow throughout.
            let mut run = race_owner.shared.run_mut();
            run.gear_inputs.first = 0;
            run.gear_inputs.second = 0x100;
            run.gear_inputs.steer = 0;
        }
        let value = race_owner.shared.car().input_keys | keys;
        race_owner.shared.car_mut().input_keys = value;
    }
    Ok(())
}
