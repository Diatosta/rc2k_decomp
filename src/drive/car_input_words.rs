//! `drive::car_input_words` (0x43A27E): the current car's input words (`input_words` .. `6138D0`):
//! 0, then, for a car with a device (`listed_car_sounds`[g_currentCar]` not -1), its readings
//! (`drive::device_axes`). In a network game (`session`): the car's block
//! (`g_carBlocks[g_currentCar]`) +0xFD0 .. gives them, or, for the car recorded (`player`),
//! takes them.
//!
//! Callers: `drive::car_controls`, `race::step_replay_cars`, `drive::replay_step`. Refuses (before
//! anything) a current car past 7, and what `device_axes` refuses.

use crate::errors::Refused;
use crate::race::owner::Race;

// Converted: `rc2k_core::drive::inputs::{check_car, device_words, replay_words}`.
pub fn car_input_words(
    race: &Race,
    input: &crate::input::owner::Input,
    net: &crate::net::owner::Net,
) -> Result<(), Refused> {
    let vars = &net.vars;
    let race_owner = race;
    let frame = &race.frame;
    use rc2k_core::drive::inputs::{NO_DEVICE, ReplayWords, check_car, device_words, replay_words};
    let car = race_owner.shared.work.current_car.get();
    check_car(car)?;
    let device = frame.frame_work.listed_car_sounds[car as usize].get();
    let readings = if device != NO_DEVICE {
        Some(crate::drive::device_axes::device_axes(device, input)?)
    } else {
        None
    };
    let words = device_words(readings);
    for (k, v) in words.iter().enumerate() {
        race_owner.shared.physics_tables.input_words[k].set(*v);
    }
    if vars.session.get() != 0 {
        let block = race_owner
            .shared
            .work
            .block_car(car as usize, "a car's block")?;
        // +0xFD0 .. +0xFDC: the kept throttle, key bits, steering and brake words.
        let kept = [
            &block.kept_throttle_word,
            &block.kept_keys_word,
            &block.kept_steering_word,
            &block.kept_brake_word,
        ];
        for (k, at) in kept.into_iter().enumerate() {
            match replay_words(car, vars.player.get()) {
                ReplayWords::FromBlock => {
                    race_owner.shared.physics_tables.input_words[k].set(at.get());
                }
                ReplayWords::IntoBlock => {
                    at.set(race_owner.shared.physics_tables.input_words[k].get());
                }
            }
        }
    }
    Ok(())
}
